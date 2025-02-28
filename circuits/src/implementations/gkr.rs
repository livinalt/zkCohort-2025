use ark_bn254::Fq;
use crate::implementations::circuit::{Circuit, Operator};
use crate::implementations::transcript::Transcript;
use crate::implementations::multilinear_polynomial::MultilinearPoly;
use super::circuit::Layers;
use super::composed_poly::{ProductPoly, SumPoly};
use super::sumcheck::{generate_sumcheck_proof, verify_sumcheck_proof};
use super::transcript::fq_vec_to_bytes;
use super::univariate_poly::UnivariatePoly;

pub struct Proof {
    final_layer_poly: MultilinearPoly<Fq>,
    sumcheck_proof_evals: Vec<Vec<UnivariatePoly<Fq>>>,
    sumcheck_claimed_evals: Vec<(Fq, Fq)>,
}

pub fn prove(circuit: &mut Circuit<Fq>, inputs: Vec<Fq>) -> Proof {
    let mut transcript = Transcript::<Fq>::init();

    let mut intermediate_layer_evals = circuit.evaluate_circuit(inputs.clone());

    let final_layer_eval = intermediate_layer_evals.last().unwrap().clone();
    let final_layer_poly = MultilinearPoly::new(final_layer_eval);

    let (sumcheck_claim, sumcheck_random_challenge) =
        initialize_sumcheck_protocol(&mut transcript, &final_layer_poly);

    let num_rounds = circuit.layers.len();
    let mut sumcheck_proof_evals = Vec::with_capacity(num_rounds);
    let mut sumcheck_claimed_evals: Vec<(Fq, Fq)> = Vec::new();

    let mut alpha_challenge = Fq::from(0);
    let mut beta_challenge = Fq::from(0);
    let mut random_challenge_b: Vec<Fq> = Vec::new();
    let mut random_challenge_c: Vec<Fq> = Vec::new();

    intermediate_layer_evals.reverse();
    let mut layers = circuit.layers.clone();
    layers.reverse();

    // Getting the evaluation polynomial for the current layer
    for (layer_index, layer) in layers.into_iter().enumerate() {
        let layer_evaluation_polynomial = if layer_index == num_rounds - 1 {
            inputs.to_vec()
        } else {
            intermediate_layer_evals[layer_index + 1].clone()
        };

        // Construct the sumcheck input polynomial
        let sumcheck_input_polynomial = if layer_index == 0 {
            construct_sumcheck_input_polynomial(
                sumcheck_random_challenge,
                layer,
                &layer_evaluation_polynomial,
                &layer_evaluation_polynomial,
            )
        } else {
            construct_merged_sumcheck_input_polynomial(
                layer,
                &layer_evaluation_polynomial,
                &layer_evaluation_polynomial,
                &random_challenge_b,
                &random_challenge_c,
                alpha_challenge,
                beta_challenge,
            )
        };

        // Generates the sumcheck proof for the current layer
        let sumcheck_proof = generate_sumcheck_proof(
            sumcheck_claim,
            &sumcheck_input_polynomial,
            &mut transcript,
        );

        // Store the proof polynomials
        sumcheck_proof_evals.push(
            sumcheck_proof.proof_polynomials
                .into_iter()
                .map(|vec| UnivariatePoly::new(vec))
                .collect(),
        );

        // Split the random challenges into r_b and r_c
        let random_challenges: Vec<Fq> = sumcheck_proof.random_challenges;
        let (r_b, r_c) = random_challenges.split_at(random_challenges.len() / 2);

        // Evaluate the layer polynomial at r_b and r_c
        let evaluation_at_rb =
            MultilinearPoly::new(layer_evaluation_polynomial.clone()).full_evaluation(r_b.to_vec());
        let evaluation_at_rc =
            MultilinearPoly::new(layer_evaluation_polynomial.clone()).full_evaluation(r_c.to_vec());

        // Update random challenges and get it ready to be absorbed into transcript
        random_challenge_b = r_b.to_vec();
        random_challenge_c = r_c.to_vec();

        transcript.absorb(&fq_vec_to_bytes(&[evaluation_at_rb]));
        alpha_challenge = transcript.squeeze();

        transcript.absorb(&fq_vec_to_bytes(&[evaluation_at_rc]));
        beta_challenge = transcript.squeeze();

        // Store-up the claimed evaluations
        sumcheck_claimed_evals.push((evaluation_at_rb, evaluation_at_rc));
    }

    Proof {
        final_layer_poly,
        sumcheck_proof_evals,
        sumcheck_claimed_evals,
    }
}

pub fn verify(proof: Proof, mut circuit: Circuit<Fq>, inputs: &[Fq]) -> bool {
    let mut transcript = Transcript::<Fq>::init();

    let (mut current_claim, init_random_challenge) =
        initialize_sumcheck_protocol(&mut transcript, &proof.final_layer_poly);

    let mut alpha_challenge = Fq::from(0);
    let mut beta_challenge = Fq::from(0);
    let mut prev_sumcheck_random_challenges = Vec::new();

    circuit.layers.reverse();
    let num_layers = circuit.layers.len();

    // Verify the sumcheck proof for the current layer
    for (layer_index, layer) in circuit.layers.iter().enumerate() {
        let sumcheck_verify = verify_sumcheck_proof(
            proof.sumcheck_proof_evals[layer_index].clone(),
            current_claim,
            &mut transcript,
        );

        if !sumcheck_verify.verified {
            return false;
        }

        // Get the random challenges for the current layer
        let current_random_challenge = sumcheck_verify.random_challenges;

        let (evaluation_at_rb, evaluation_at_rc) = if layer_index == num_layers - 1 {
            evaluate_input_layer_polynomial(inputs, &current_random_challenge)
        } else {
            proof.sumcheck_claimed_evals[layer_index]
        };

        // Compute the verifier's claim for the current layer
        let expected_claim = if layer_index == 0 {
            compute_verifier_claim_for_layer(
                layer,
                init_random_challenge,
                &current_random_challenge,
                evaluation_at_rb,
                evaluation_at_rc,
            )
        } else {
            compute_merged_verifier_claim(
                layer,
                &current_random_challenge,
                &prev_sumcheck_random_challenges,
                evaluation_at_rb,
                evaluation_at_rc,
                alpha_challenge,
                beta_challenge,
            )
        };

        // Check if the verifier's claim matches the sumcheck result
        if expected_claim != sumcheck_verify.final_claimed_sum {
            return false;
        }

        // Update previous random challenges and absorb into the transcript
        prev_sumcheck_random_challenges = current_random_challenge;

        transcript.absorb(&fq_vec_to_bytes(&[evaluation_at_rb]));
        alpha_challenge = transcript.squeeze();

        transcript.absorb(&fq_vec_to_bytes(&[evaluation_at_rc]));
        beta_challenge = transcript.squeeze();

        current_claim = (alpha_challenge * evaluation_at_rb) + (beta_challenge * evaluation_at_rc);
    }

    true
}

fn initialize_sumcheck_protocol(
    transcript: &mut Transcript<Fq>,
    final_layer_poly: &MultilinearPoly<Fq>,
) -> (Fq, Fq) {
    transcript.absorb(&fq_vec_to_bytes(&final_layer_poly.evaluation));

    let random_challenge = transcript.squeeze();

    let m_0 = final_layer_poly.full_evaluation(vec![random_challenge]);

    transcript.absorb(&fq_vec_to_bytes(&[m_0]));

    (m_0, random_challenge)
}

fn combine_polynomials_using_operators(
    poly_a: &[Fq],
    poly_b: &[Fq],
    op: Operator,
) -> MultilinearPoly<Fq> {
    let new_eval: Vec<Fq> = poly_a
        .iter()
        .flat_map(|a| poly_b.iter().map(move |b| op.use_operation(*a, *b)))
        .collect();

    MultilinearPoly::new(new_eval)
}

fn construct_sumcheck_input_polynomial(
    random_challenge: Fq,
    layer: Layers<Fq>,
    w_b: &[Fq],
    w_c: &[Fq],
) -> SumPoly<Fq> {
    let add_i = layer
        .get_add_mul_i(Operator::Add)
        .partial_evaluate(0, &random_challenge);
    let mul_i = layer
        .get_add_mul_i(Operator::Mul)
        .partial_evaluate(0, &random_challenge);

    let summed_w_poly = combine_polynomials_using_operators(w_b, w_c, Operator::Add);
    let multiplied_w_poly = combine_polynomials_using_operators(w_b, w_c, Operator::Mul);

    let add_eval_product = ProductPoly::init_poly(vec![add_i.evaluation, summed_w_poly.evaluation]);
    let mul_eval_product = ProductPoly::init_poly(vec![mul_i.evaluation, multiplied_w_poly.evaluation]);

    SumPoly::new(vec![add_eval_product, mul_eval_product])
}

fn construct_merged_sumcheck_input_polynomial(
    layer: Layers<Fq>,
    w_b: &[Fq],
    w_c: &[Fq],
    r_b: &[Fq],
    r_c: &[Fq],
    alpha: Fq,
    beta: Fq,
) -> SumPoly<Fq> {
    let add_i = layer.get_add_mul_i(Operator::Add);
    let mul_i = layer.get_add_mul_i(Operator::Mul);

    let summed_add_i = add_i.multi_partial_evaluate(r_b).scale(alpha)
        + add_i.multi_partial_evaluate(r_c).scale(beta);

    let summed_mul_i = mul_i.multi_partial_evaluate(r_b).scale(alpha)
        + mul_i.multi_partial_evaluate(r_c).scale(beta);

    let summed_w_poly = combine_polynomials_using_operators(w_b, w_c, Operator::Add);
    let multiplied_w_poly = combine_polynomials_using_operators(w_b, w_c, Operator::Mul);

    let add_product_poly =
        ProductPoly::init_poly(vec![summed_add_i.evaluation, summed_w_poly.evaluation]);
    let mul_product_poly =
        ProductPoly::init_poly(vec![summed_mul_i.evaluation, multiplied_w_poly.evaluation]);

    SumPoly::new(vec![add_product_poly, mul_product_poly])
}

fn compute_verifier_claim_for_layer(
    layer: &Layers<Fq>,
    init_random_challenge: Fq,
    sumcheck_random_challenges: &[Fq],
    o_1: Fq,
    o_2: Fq,
) -> Fq {
    let (r_b, r_c) = sumcheck_random_challenges.split_at(sumcheck_random_challenges.len() / 2);
    let mut all_random_challenges = Vec::with_capacity(1 + r_b.len() + r_c.len());
    all_random_challenges.push(init_random_challenge);
    all_random_challenges.extend_from_slice(r_b);
    all_random_challenges.extend_from_slice(r_c);

    let a_r = layer
        .get_add_mul_i(Operator::Add)
        .full_evaluation(all_random_challenges.clone());
    let m_r = layer
        .get_add_mul_i(Operator::Mul)
        .full_evaluation(all_random_challenges);
    (a_r * (o_1 + o_2)) + (m_r * (o_1 * o_2))
}

fn compute_merged_verifier_claim(
    layer: &Layers<Fq>,
    current_random_challenge: &[Fq],
    previous_random_challenge: &[Fq],
    o_1: Fq,
    o_2: Fq,
    alpha: Fq,
    beta: Fq,
) -> Fq {
    let (prev_r_b, prev_r_c) =
        previous_random_challenge.split_at(previous_random_challenge.len() / 2);

    let add_i = layer.get_add_mul_i(Operator::Add);
    let mul_i = layer.get_add_mul_i(Operator::Mul);

    let summed_add_i = add_i.multi_partial_evaluate(prev_r_b).scale(alpha)
        + add_i.multi_partial_evaluate(prev_r_c).scale(beta);

    let summed_mul_i = mul_i.multi_partial_evaluate(prev_r_b).scale(alpha)
        + mul_i.multi_partial_evaluate(prev_r_c).scale(beta);

    let a_r = summed_add_i.full_evaluation(current_random_challenge.to_vec());
    let m_r = summed_mul_i.full_evaluation(current_random_challenge.to_vec());

    (a_r * (o_1 + o_2)) + (m_r * (o_1 * o_2))
}

fn evaluate_input_layer_polynomial(inputs: &[Fq], sumcheck_random_challenges: &[Fq]) -> (Fq, Fq) {
    let input_poly = MultilinearPoly::new(inputs.to_vec());

    let (r_b, r_c) = sumcheck_random_challenges.split_at(sumcheck_random_challenges.len() / 2);

    let o_1 = input_poly.full_evaluation(r_b.to_vec());
    let o_2 = input_poly.full_evaluation(r_c.to_vec());

    (o_1, o_2)
}


#[cfg(test)]
mod test {
    use ark_bn254::Fq;
    use super::{
        prove, verify, combine_polynomials_using_operators, construct_sumcheck_input_polynomial,
    };
    use crate::implementations::{
        circuit::{Circuit, Gates, Layers, Operator},
        multilinear_polynomial::MultilinearPoly,
        univariate_poly::UnivariatePoly,
        composed_poly::{ProductPoly, SumPoly},
    };

    #[test]
    fn it_add_polys_correctly() {
        let poly_a = &[Fq::from(0), Fq::from(2)];
        let poly_b = &[Fq::from(0), Fq::from(3)];

        let expected_poly = vec![Fq::from(0), Fq::from(3), Fq::from(2), Fq::from(5)];

        let result = combine_polynomials_using_operators(poly_a, poly_b, Operator::Add);

        assert_eq!(result.evaluation, expected_poly);

        let poly_a = &[Fq::from(0), Fq::from(3)];
        let poly_b = &[Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)];

        let expected_poly = vec![
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(2),
            Fq::from(3),
            Fq::from(3),
            Fq::from(3),
            Fq::from(5),
        ];

        let result = combine_polynomials_using_operators(poly_a, poly_b, Operator::Add);

        assert_eq!(result.evaluation, expected_poly);
    }

    #[test]
    fn it_multiplies_polys_correctly() {
        let poly_a = &[Fq::from(0), Fq::from(2)];
        let poly_b = &[Fq::from(0), Fq::from(3)];

        let expected_poly = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(6)];

        let result = combine_polynomials_using_operators(poly_a, poly_b, Operator::Mul);

        assert_eq!(result.evaluation, expected_poly);

        let poly_a = &[Fq::from(0), Fq::from(3)];
        let poly_b = &[Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)];

        let expected_poly = vec![
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(0),
            Fq::from(6),
        ];

        let result = combine_polynomials_using_operators(poly_a, poly_b, Operator::Mul);

        assert_eq!(result.evaluation, expected_poly);
    }

    #[test]
    fn test_construct_sumcheck_input_polynomial() {
        let gate = Gates::new_gate(Fq::from(2), Fq::from(14), Operator::Add);

        let layer = Layers::new_layer(vec![gate]);

        let r_c = Fq::from(5);

        let w_1_poly = &[Fq::from(2), Fq::from(12)];

        let add_i_r =
            MultilinearPoly::new(vec![Fq::from(0), Fq::from(-4), Fq::from(0), Fq::from(0)]);

        let mul_i_r = MultilinearPoly::new(vec![Fq::from(0); 4]);

        let fbc_poly = construct_sumcheck_input_polynomial(r_c, layer, w_1_poly, w_1_poly);

        let one = ProductPoly::init_poly(vec![
            add_i_r.evaluation,
            vec![Fq::from(4), Fq::from(14), Fq::from(14), Fq::from(24)],
        ]);

        let two = ProductPoly::init_poly(vec![
            mul_i_r.evaluation,
            vec![Fq::from(4), Fq::from(24), Fq::from(24), Fq::from(144)],
        ]);

        let expected_result = SumPoly::new(vec![one, two]);

        assert_eq!(fbc_poly.polys, expected_result.polys);
    }

    #[test]
    fn test_prove_and_verify() {
        let gate = Gates::new_gate(Fq::from(2), Fq::from(14), Operator::Add);
        let layer = Layers::new_layer(vec![gate]);
        let mut circuit = Circuit::new_circuit(vec![layer]);

        let inputs = vec![Fq::from(2), Fq::from(3)];

        let proof = prove(&mut circuit, inputs.clone());

        let is_valid = verify(proof, circuit, &inputs);

        assert!(is_valid);
    }
}