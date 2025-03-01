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

    // Evaluate the circuit on the given inputs
    let mut circuit_evaluations = circuit.evaluate_circuit(inputs.clone());

    println!("This is the Circuit Evaluation {:?}", circuit_evaluations);

    // Handle the case where the final layer evaluation has a single value
    let mut w_0 = circuit_evaluations.last().unwrap().clone();

    println!("This is the first Layer W_0 {:?}", w_0);

    if w_0.len() == 1 {
        w_0.push(Fq::from(0)); // Pad with zero to make length a power of two
    }

        println!("After blowing up first Layer W_0 {:?}", w_0);


        // ++++++++++++++++++++++++++++++++++++FAILING HERE WHERE MULTILINEAR IS CALLED ++++++++++++++++++++++++++++++++++++++++

    // Create the output polynomial from the padded evaluation
    let output_poly = MultilinearPoly::new(w_0);
    println!("This is the Output_pol Created {:?}", output_poly);

    // Initialize the sumcheck protocol
    let (mut claimed_sum, random_challenge) = initialize_sumcheck_protocol(&mut transcript, &output_poly);

    println!("This is the Claimed Sum {:?} and Random Challenge {:?}", claimed_sum, random_challenge);

    let num_layers = circuit.layers.len();
    let mut proof_polynomials = Vec::with_capacity(num_layers);
    let mut claimed_evaluations = Vec::with_capacity(num_layers - 1);
    let mut current_rb = Vec::new();
    let mut current_rc = Vec::new();
    let mut alpha = Fq::from(0);
    let mut beta = Fq::from(0);

    // println!("This is the Proof Polynomials {:?}", proof_polynomials);
    // println!("This is the Number of Layers {:?}", num_layers);
    // println!("This is the Claim Evaluations {:?}", claimed_evaluations);
    // println!("This is the Circuit Layers {:?}", circuit.layers);
    // println!("This is the Circuit Evaluations {:?}", circuit_evaluations);

    // Reverse the intermediate evaluations and layers for processing
    circuit_evaluations.reverse();
    let mut layers = circuit.layers.clone();
    layers.reverse();

    // Process each layer
    for (idx, layer) in layers.into_iter().enumerate() {
        // Get the evaluation polynomial for the current layer
        let w_i = if idx == num_layers - 1 {
            inputs.to_vec() // For the input layer, use the inputs directly
        } else {
            circuit_evaluations[idx + 1].clone() 
        };

        // Construct the sumcheck input polynomial
        let fbc_poly = if idx == 0 {
            construct_sumcheck_input_polynomial(random_challenge, layer, &w_i, &w_i)
        } else {
            construct_merged_sumcheck_input_polynomial(layer, &w_i, &w_i, &current_rb, &current_rc, alpha, beta)
        };

        // Generate the sumcheck proof for the current layer
        let sum_check_proof = generate_sumcheck_proof(claimed_sum, &fbc_poly, &mut transcript);

        // Store the proof polynomials
        proof_polynomials.push(
            sum_check_proof.proof_polynomials
                .into_iter()
                .map(|vec| UnivariatePoly::new(vec))
                .collect(),
        );

        if idx < num_layers - 1 {
            // Evaluate the layer polynomial at r_b and r_c
            let next_poly = MultilinearPoly::new(w_i);
            let mid = sum_check_proof.random_challenges.len() / 2;
            let (r_b, r_c) = sum_check_proof.random_challenges.split_at(mid);

            let o_1 = next_poly.full_evaluation(r_b.to_vec());
            let o_2 = next_poly.full_evaluation(r_c.to_vec());
            current_rb = r_b.to_vec();
            current_rc = r_c.to_vec();

            // Update the transcript and challenges
            transcript.absorb(&fq_vec_to_bytes(&[o_1]));
            alpha = transcript.squeeze();

            transcript.absorb(&fq_vec_to_bytes(&[o_2]));
            beta = transcript.squeeze();

            // Update the claimed sum for the next layer
            claimed_sum = (alpha * o_1) + (beta * o_2);
            claimed_evaluations.push((o_1, o_2));
        }
    }

    // Return the proof
    Proof {
        final_layer_poly: output_poly,
        sumcheck_proof_evals: proof_polynomials,
        sumcheck_claimed_evals: claimed_evaluations,
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
        prove, verify, combine_polynomials_using_operators,
    };
    use crate::implementations::
        circuit::{Circuit, Gates, Layers, Operator}
    ;

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
    fn test_prove() {
        let inputs: Vec<Fq> = vec![
            Fq::from(5),
            Fq::from(2),
            Fq::from(2),
            Fq::from(4),
            Fq::from(10),
            Fq::from(0),
            Fq::from(3),
            Fq::from(3),
        ];

        let test_circuit: Vec<Layers<Fq>> = vec![
            Layers::new_layer(vec![
                Gates::new_gate(inputs[0], inputs[1], Operator::Mul),
                Gates::new_gate(inputs[2], inputs[3], Operator::Mul),
                Gates::new_gate(inputs[4], inputs[5], Operator::Mul),
                Gates::new_gate(inputs[6], inputs[7], Operator::Mul),
            ]),
            Layers::new_layer(vec![
                Gates::new_gate(inputs[0], inputs[1], Operator::Add),
                Gates::new_gate(inputs[2], inputs[3], Operator::Add),
            ]),
            Layers::new_layer(vec![
                Gates::new_gate(inputs[4], inputs[5], Operator::Add),
            ]),
        ];

        let mut circuit = Circuit::<Fq>::new_circuit(test_circuit);

        // Generate proof
        let proof = prove(&mut circuit, inputs.clone());

        // Verify proof
        let verification_result = verify(proof, circuit, &inputs);

        assert!(verification_result, "Proof verification failed");
    }

//     #[test]
// fn test_prove_verify() {
//     let inputs: Vec<Fq> = vec![
//         Fq::from(5),
//         Fq::from(2),
//         Fq::from(2),
//         Fq::from(4),
//         Fq::from(10),
//         Fq::from(0),
//         Fq::from(3),
//         Fq::from(3),
//     ];

//     let test_circuit: Vec<Layers<Fq>> = vec![
//         Layers::new_layer(vec![
//             Gates::new_gate(inputs[0], inputs[1], Operator::Mul),
//             Gates::new_gate(inputs[2], inputs[3], Operator::Mul),
//             Gates::new_gate(inputs[4], inputs[5], Operator::Mul),
//             Gates::new_gate(inputs[6], inputs[7], Operator::Mul),
//         ]),
//         Layers::new_layer(vec![
//             Gates::new_gate(inputs[0], inputs[1], Operator::Add),
//             Gates::new_gate(inputs[2], inputs[3], Operator::Add),
//         ]),
//         Layers::new_layer(vec![
//             Gates::new_gate(inputs[4], inputs[5], Operator::Add),
//         ]),
//     ];

//     let mut circuit = Circuit::<Fq>::new_circuit(test_circuit);

//     // Generate proof
//     let proof = prove(&mut circuit, inputs.clone());

//     // Verify proof
//     let verification_result = verify(proof, circuit, &inputs);

//     assert!(verification_result, "Proof verification failed");



//     }


//  #[test]
//     fn test_valid_proving_and_verification() {
//         let circuit_structure: Vec<Layers<Fq>> = vec![
//             Layers::new_layer(vec![
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Mul),
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Mul),
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Mul),
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Mul),
//             ]),
//             Layers::new_layer(vec![
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Add),
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Add),
//             ]),
//             Layers::new_layer(vec![
//                 Gates::new_gate(Fq::from(0), Fq::from(0), Operator::Add),
//             ]),
//         ];

//         let inputs: Vec<Fq> = vec![
//             Fq::from(5),
//             Fq::from(2),
//             Fq::from(2),
//             Fq::from(4),
//             Fq::from(10),
//             Fq::from(0),
//             Fq::from(3),
//             Fq::from(3),
//         ];

//         let mut circuit = Circuit::new_circuit(circuit_structure);

//         let proof = prove(&mut circuit, inputs.clone());

//         let is_verified = verify(proof, circuit, &inputs);

//         assert_eq!(is_verified, true);
//     }

}