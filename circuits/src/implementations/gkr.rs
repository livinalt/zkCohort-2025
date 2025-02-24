use ark_bn254::Fq;
use crate::implementations::circuit::{Circuit, Operator};

use crate::implementations::transcript::Transcript;
use crate::implementations::multilinear_polynomial::MultilinearPoly;
use super::circuit::Layers;
use super::composed_poly::{ProductPoly, SumPoly};
use super::sumcheck::{gkr_prove, gkr_verify};
use super::transcript::fq_vec_to_bytes;
use super::univariate_poly::UnivariatePoly;

pub struct Proof {
    output_poly: MultilinearPoly<Fq>,
    proof_polynomials: Vec<Vec<UnivariatePoly<Fq>>>,
    claimed_evaluations: Vec<(Fq, Fq)>,
}


pub fn prove(circuit: &mut Circuit<Fq>, inputs: Vec<Fq>) -> Proof {
    let mut transcript = Transcript::<Fq>::init();

    let mut circuit_evaluations = circuit.evaluate_circuit(inputs.clone());

    let mut w_0 = circuit_evaluations.last().unwrap().clone();

    if w_0.len() == 1 {
        w_0.push(Fq::from(0));
    }

    let output_poly = MultilinearPoly::new(w_0);

let (mut claimed_sum, mut random_challenge) = initiate_protocol(&mut transcript, &output_poly);


    let num_rounds = circuit.layers.len();
    let mut proof_polys = Vec::with_capacity(num_rounds);
    let mut claimed_evaluations: Vec<(Fq, Fq)> = Vec::new();

    let mut current_alpha = Fq::from(0);
    let mut current_beta = Fq::from(0);
    let mut current_rb: Vec<Fq> = Vec::new();
    let mut current_rc: Vec<Fq> = Vec::new();

    circuit_evaluations.reverse();

        let mut layers = circuit.layers.clone();
    layers.reverse();

     for (idx, layer) in layers.into_iter().enumerate() {
        let w_i = if idx == num_rounds - 1 {
            inputs.to_vec()
        } else {
            circuit_evaluations[idx + 1].clone()
        };

        let fbc_poly = if idx == 0 {
            get_fbc_poly(random_challenge, layer, &w_i, &w_i)
        } else {
            get_merged_fbc_poly(layer, &w_i, &w_i, &current_rb, &current_rc, current_alpha, current_beta)
        };
     
        let sum_check_proof = gkr_prove(claimed_sum, &fbc_poly, &mut transcript);

        proof_polys.push(
            sum_check_proof.proof_polynomials
                .into_iter()
                .map(|vec| UnivariatePoly::new(vec))
                .collect()
        );

        let random_challenges: Vec<ark_ff::Fp<ark_ff::MontBackend<ark_bn254::FqConfig, 4>, 4>> = sum_check_proof.random_challenges;

        let (r_b, r_c) = random_challenges.split_at(random_challenges.len() / 2);

        let o_1 = MultilinearPoly::new(w_i.clone()).full_evaluation(r_b.to_vec());
        let o_2 = MultilinearPoly::new(w_i.clone()).full_evaluation(r_c.to_vec());

        current_rb = r_b.to_vec();
        current_rc = r_c.to_vec();

        transcript.absorb(&fq_vec_to_bytes(&[o_1]));
        current_alpha = transcript.squeeze();

        transcript.absorb(&fq_vec_to_bytes(&[o_2]));
        current_beta = transcript.squeeze();

        claimed_evaluations.push((o_1, o_2));

        random_challenge = transcript.squeeze();
    }

    Proof {
        output_poly,
        proof_polynomials: proof_polys,
        claimed_evaluations,
    }
}

pub fn verify(proof: Proof, mut circuit: Circuit<Fq>, inputs: &[Fq]) -> bool {
    let mut transcript = Transcript::<Fq>::init();

    let (mut current_claim, init_random_challenge) =
        initiate_protocol(&mut transcript, &proof.output_poly);

    let mut alpha = Fq::from(0);
    let mut beta = Fq::from(0);
    let mut prev_sumcheck_random_challenges = Vec::new();

    circuit.layers.reverse();
    let num_layers = circuit.layers.len();

    for (i, layer) in circuit.layers.iter().enumerate() {
        let sum_check_verify = gkr_verify(
            proof.proof_polynomials[i].clone(),
            current_claim,
            &mut transcript,
        );

        if !sum_check_verify.verified {
            return false;
        }

        let current_random_challenge = sum_check_verify.random_challenges;

        let (o_1, o_2) = if i == num_layers - 1 {
            evaluate_input_poly(inputs, &current_random_challenge)
        } else {
            proof.claimed_evaluations[i]
        };

        let expected_claim = if i == 0 {
            get_verifier_claim(
                layer,
                init_random_challenge,
                &current_random_challenge,
                o_1,
                o_2,
            )
        } else {
            get_merged_verifier_claim(
                layer,
                &current_random_challenge,
                &prev_sumcheck_random_challenges,
                o_1,
                o_2,
                alpha,
                beta,
            )
        };

        if expected_claim != sum_check_verify.final_claimed_sum {
            return false;
        }

        prev_sumcheck_random_challenges = current_random_challenge;

        transcript.absorb(&fq_vec_to_bytes(&[o_1]));
        alpha = transcript.squeeze();

        transcript.absorb(&fq_vec_to_bytes(&[o_2]));
        beta = transcript.squeeze();

        current_claim = (alpha * o_1) + (beta * o_2);
    }

    true
}


fn initiate_protocol(
    transcript: &mut Transcript<Fq>,
    output_poly: &MultilinearPoly<Fq>,
) -> (Fq, Fq) {
    transcript.absorb(&fq_vec_to_bytes(&output_poly.evaluation));

    let random_challenge = transcript.squeeze();

    let m_0 = output_poly.full_evaluation(vec![random_challenge]);

    transcript.absorb(&fq_vec_to_bytes(&[m_0]));

    (m_0, random_challenge)
}

fn tensor_add_mul_polynomials(poly_a: &[Fq], poly_b: &[Fq], op: Operator) -> MultilinearPoly<Fq> {
    let new_eval: Vec<Fq> = poly_a
        .iter()
        .flat_map(|a| poly_b.iter().map(move |b| op.use_operation(*a, *b)))
        .collect();

    MultilinearPoly::new(new_eval)
}



pub fn get_fbc_poly(random_challenge: Fq, layer: Layers<Fq>, w_b: &[Fq], w_c: &[Fq]) -> SumPoly<Fq> {
    let add_i = layer
        .get_add_mul_i(Operator::Add)
        .partial_evaluate(0, &random_challenge);
    let mul_i = layer
        .get_add_mul_i(Operator::Mul)
        .partial_evaluate(0, &random_challenge);

    let summed_w_poly = tensor_add_mul_polynomials(w_b, w_c, Operator::Add);
    let multiplied_w_poly = tensor_add_mul_polynomials(w_b, w_c, Operator::Mul);

    let add_eval_product = ProductPoly::new(vec![add_i.evaluation, summed_w_poly.evaluation]);
    let mul_eval_product = ProductPoly::new(vec![mul_i.evaluation, multiplied_w_poly.evaluation]);

    SumPoly::new(vec![add_eval_product, mul_eval_product])
}

fn get_merged_fbc_poly(
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

    let summed_w_poly = tensor_add_mul_polynomials(w_b, w_c, Operator::Add);
    let multiplied_w_poly = tensor_add_mul_polynomials(w_b, w_c, Operator::Mul);

    let add_product_poly =
        ProductPoly::new(vec![summed_add_i.evaluation, summed_w_poly.evaluation]);
    let mul_product_poly =
        ProductPoly::new(vec![summed_mul_i.evaluation, multiplied_w_poly.evaluation]);

    SumPoly::new(vec![add_product_poly, mul_product_poly])
}

fn get_verifier_claim(
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

fn get_merged_verifier_claim(
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

fn evaluate_input_poly(inputs: &[Fq], sumcheck_random_challenges: &[Fq]) -> (Fq, Fq) {
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
        get_fbc_poly, get_merged_fbc_poly, prove, tensor_add_mul_polynomials, verify, Proof,
    };
    use crate::implementations::{circuit::{Circuit, Gates, Layers, Operator}, multilinear_polynomial, univariate_poly};
    use crate::implementations::multilinear_polynomial::MultilinearPoly;
    use crate::implementations::univariate_poly::UnivariatePoly;
    use crate::implementations::composed_poly::{ProductPoly, SumPoly};


    #[test]
    fn it_add_polys_correctly() {
        let poly_a = &[Fq::from(0), Fq::from(2)];
        let poly_b = &[Fq::from(0), Fq::from(3)];

        let expected_poly = vec![Fq::from(0), Fq::from(3), Fq::from(2), Fq::from(5)];

        let result = tensor_add_mul_polynomials(poly_a, poly_b, Operator::Add);

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

        let result = tensor_add_mul_polynomials(poly_a, poly_b, Operator::Add);

        assert_eq!(result.evaluation, expected_poly);
    }

    #[test]
    fn it_multiplies_polys_correctly() {
        let poly_a = &[Fq::from(0), Fq::from(2)];
        let poly_b = &[Fq::from(0), Fq::from(3)];

        let expected_poly = vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(6)];

        let result = tensor_add_mul_polynomials(poly_a, poly_b, Operator::Mul);

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

        let result = tensor_add_mul_polynomials(poly_a, poly_b, Operator::Mul);

        assert_eq!(result.evaluation, expected_poly);
    }

    #[test]
    fn test_get_fbc_poly() {
        let gate = Gates::new_gate(Fq::from(2), Fq::from(14), Operator::Add);

        let layer = Layers::new_layer(vec![gate]);

        let r_c = Fq::from(5);

        let w_1_poly = &[Fq::from(2), Fq::from(12)];

        let add_i_r =
            MultilinearPoly::new(vec![Fq::from(0), Fq::from(-4), Fq::from(0), Fq::from(0)]);

        let mul_i_r = MultilinearPoly::new(vec![Fq::from(0); 4]);

        let fbc_poly = get_fbc_poly(r_c, layer, w_1_poly, w_1_poly);

        let one = ProductPoly::new(vec![
            add_i_r.evaluation,
            vec![Fq::from(4), Fq::from(14), Fq::from(14), Fq::from(24)],
        ]);

        let two = ProductPoly::new(vec![
            mul_i_r.evaluation,
            vec![Fq::from(4), Fq::from(24), Fq::from(24), Fq::from(144)],
        ]);

        let expected_result = SumPoly::new(vec![one, two]);

        assert_eq!(fbc_poly.polys, expected_result.polys);
    }


}

