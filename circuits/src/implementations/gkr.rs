use ark_bn254::Fq;
use crate::implementations::Circuit::{Circuit, Operator};

use crate::implementations::fiat_shamir::{verify};
use crate::implementations::transcript::Transcript;
use crate::implementations::multilinear_polynomial::MultilinearPoly;

use super::composed_poly::SumPoly;
// use sum_check::sum_check_protocol::{gkr_prove, gkr_verify};
pub struct Proof {
    output_poly: MultilinearPoly<Fq>,
    claimed_sum: Fq,
    proof_polynomials: Vec<Vec<Vec<Fq>>>,
    claimed_evaluations: Vec<(Fq, Fq)>,
}

pub fn prove(circuit: &mut Circuit<Fq>, inputs: Vec<Fq>) -> Proof {
    let mut transcript = Transcript::<Fq>::new();

    let mut circuit_evaluations = circuit.evaluate(inputs);

    let mut w_0 = circuit_evaluations.last().unwrap().to_vec();

    if w_0.len() == 1 {
        w_0.push(Fq::from(0));
    }

    let output_poly = MultilinearPoly::new(w_0);

    let (m_0, r_c) = initiate_protocol(&mut transcript, &output_poly);

    let claimed_sum = m_0;

    let mut random_challenge = r_c;

    let num_rounds = 2;
    let mut proof_polys = Vec::with_capacity(num_rounds);
    let mut claimed_evaluations: Vec<(Fq, Fq)> = Vec::new();

    let mut current_alpha = Fq::from(0);
    let mut current_beta = Fq::from(0);
    let mut current_rb = Fq::from(0);
    let mut current_rc = Fq::from(0);

    circuit_evaluations.reverse();

    for idx in 0..circuit.layers.len() {
        let mut layers = circuit.layers.clone();
        layers.reverse();

        let add_i = layers[idx].get_add_mul_i(Operator::Add);
        let mul_i = layers[idx].get_add_mul_i(Operator::Mul);

        let w_i = if idx + 1 < circuit.layers.len() {
            circuit_evaluations[idx + 1].clone()
        } else {
            inputs.clone()
        };

        let fbc_poly: SumPoly<Fq> = if idx == 0 {
            get_fbc_poly(random_challenge, add_i, mul_i, &w_i, &w_i)
            ////! ////////////correct up to here
        } else {
            ////! this throws the error
            get_merged_fbc_poly(
                add_i,
                mul_i,
                &w_i,
                &w_i,
                current_rb,
                current_rc,
                current_alpha,
                current_beta,
            )
        };

        let sum_check_proof = gkr_prove(claimed_sum, &fbc_poly, &mut transcript);

        proof_polys.push(sum_check_proof.proof_polynomials);

        let next_poly = MultilinearPoly::new(w_i);

        let o_1 = next_poly.evaluate(vec![sum_check_proof.random_challenges[0]]);
        let o_2 = next_poly.evaluate(vec![sum_check_proof.random_challenges[1]]);

        current_rb = o_1;
        current_rc = o_2;

        transcript.append(&fq_vec_to_bytes(&[o_1]));
        current_alpha = transcript.get_random_challenge();

        transcript.append(&fq_vec_to_bytes(&[o_2]));
        current_beta = transcript.get_random_challenge();

        claimed_evaluations.push((o_1, o_2));

        random_challenge = transcript.get_random_challenge();
    }

    Proof {
        output_poly,
        claimed_sum: m_0,
        proof_polynomials: proof_polys,
        claimed_evaluations,
    }
}

fn verify(proof: Proof, circuit: Circuit<Fq>) -> bool {
    let mut transcript = Transcript::<Fq>::new();

    let (_, _) = initiate_protocol(&mut transcript, &proof.output_poly);

    let mut current_claim = proof.claimed_sum;

    for i in 0..circuit.layers.len() {
        let sum_check_verify = gkr_verify(
            proof.proof_polynomials[i].clone(),
            current_claim,
            transcript.clone(),
        );

        if !sum_check_verify.verified {
            return false;
        }

        let final_claimed_sum = sum_check_verify.final_claimed_sum;

        let layers = &circuit.layers;
        let add_i = layers[i].get_add_mul_i(Operation::Add);
        let mul_i = layers[i].get_add_mul_i(Operation::Mul);

        let r_c = sum_check_verify.random_challenges;

        let mut a_r = add_i.evaluate(r_c.clone());
        let mut m_r = mul_i.evaluate(r_c);

        let (o_1, o_2) = proof.claimed_evaluations[i];

        transcript.append(&fq_vec_to_bytes(&[o_1]));
        let alpha = transcript.get_random_challenge();

        transcript.append(&fq_vec_to_bytes(&[o_2]));
        let beta = transcript.get_random_challenge();

        if i > 0 {
            a_r *= alpha;
            m_r *= beta;
        }

        let next_layer_claim_poly = a_r * (o_1 + o_2) + m_r * (o_1 * o_2);

        if next_layer_claim_poly == final_claimed_sum {
            current_claim = (alpha * o_1) + (beta * o_2);
        }
    }

    //this is when verifier reaches the last layer (input layer), to be true if false is never returned
    return true;
}

fn initiate_protocol(
    transcript: &mut Transcript<Fq>,
    output_poly: &MultilinearPoly<Fq>,
) -> (Fq, Fq) {
    transcript.append(&fq_vec_to_bytes(&output_poly.evaluation));

    let random_challenge = transcript.get_random_challenge();

    let m_0 = output_poly.evaluate(vec![random_challenge]);

    transcript.append(&fq_vec_to_bytes(&[m_0]));

    (m_0, random_challenge)
}

fn add_mul_polynomials(poly_a: &[Fq], poly_b: &[Fq], op: Operation) -> MultilinearPoly<Fq> {
    let new_eval_len = poly_a.len() * poly_b.len();
    let mut new_eval = Vec::with_capacity(new_eval_len);

    for a in poly_a {
        for b in poly_b {
            new_eval.push(op.apply(*a, *b))
        }
    }

    MultilinearPoly::new(new_eval)
}

////! verified as correct
fn get_fbc_poly(
    random_challenge: Fq,
    add_i: MultilinearPoly<Fq>,
    mul_i: MultilinearPoly<Fq>,
    w_b: &[Fq],
    w_c: &[Fq],
) -> SumPoly<Fq> {
    let new_add_i = add_i.partial_evaluate(0, &random_challenge);
    let new_mul_i = mul_i.partial_evaluate(0, &random_challenge);

    let summed_w_poly = add_mul_polynomials(w_b, w_c, Operator::Add);
    let multiplied_w_poly = add_mul_polynomials(w_b, w_c, Operator::Mul);

    let add_w_eval = vec![new_add_i.evaluation, summed_w_poly.evaluation];
    let mul_w_eval = vec![new_mul_i.evaluation, multiplied_w_poly.evaluation];

    let add_eval_product = ProductPoly::new(add_w_eval);
    let mul_eval_product = ProductPoly::new(mul_w_eval);

    SumPoly::new(vec![add_eval_product, mul_eval_product])
}

fn get_merged_fbc_poly(
    add_i: MultilinearPoly<Fq>,
    mul_i: MultilinearPoly<Fq>,
    w_b: &[Fq],
    w_c: &[Fq],
    r_b: Fq,
    r_c: Fq,
    alpha: Fq,
    beta: Fq,
) -> SumPoly<Fq> {
    let add_i_rb = add_i.partial_evaluate(0, &r_b).scale(alpha);

    let add_i_rc = add_i.partial_evaluate(0, &r_c).scale(beta);

    let mul_i_rb = mul_i.partial_evaluate(0, &r_b).scale(alpha);

    let mul_i_rc = mul_i.partial_evaluate(0, &r_c).scale(beta);

    let summed_w_poly = add_mul_polynomials(w_b, w_c, Operation::Add);
    let multiplied_w_poly = add_mul_polynomials(w_b, w_c, Operation::Mul);

    let summed_add_i = add_i_rb + add_i_rc;

    let summed_mul_i = mul_i_rb + mul_i_rc;

    let add_product_poly = ProductPoly::new(vec![
        summed_add_i.evaluation,
        summed_w_poly.evaluation.clone(),
    ]);

    let mul_product_poly = ProductPoly::new(vec![
        summed_mul_i.evaluation,
        multiplied_w_poly.evaluation.clone(),
    ]);

    SumPoly::new(vec![add_product_poly, mul_product_poly])
}
