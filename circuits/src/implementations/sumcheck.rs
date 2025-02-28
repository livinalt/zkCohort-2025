use crate::implementations::multilinear_polynomial::MultilinearPoly;
use crate::implementations::transcript::{Transcript};
use crate::implementations::univariate_poly::UnivariatePoly;
use ark_ff::{BigInteger, PrimeField};
use ark_bn254::{Fq, Fr};
use sha3::{Keccak256, Digest};

use super::composed_poly::{ProductPoly, SumPoly};
use super::transcript::fq_vec_to_bytes;

#[derive(Debug, Clone)]
pub struct Proof {
    claimed_sum: Fq,
    proof_polynomials: Vec<Vec<Fq>>,
}

pub struct GkrProof {
    pub proof_polynomials: Vec<Vec<Fq>>,
    pub claimed_sum: Fq,
    pub random_challenges: Vec<Fq>,
}

pub struct GkrVerify {
    pub verified: bool,
    pub final_claimed_sum: Fq,
    pub random_challenges: Vec<Fq>,
}


// Helper function to computes the partial sums of of the poly
pub fn partial_sum_proof(polynomial: &[Fq]) -> Vec<Fq> {
    let split_poly_length = polynomial.len() / 2;

    let f_0 = &polynomial[..split_poly_length]; 
    let f_1 = &polynomial[split_poly_length..]; 

    let mut s_0 = Fq::from(0);
    for i in 0..f_0.len() {
        s_0 += f_0[i];
    }

    let mut s_1 = Fq::from(0); 
    for i in 0..f_1.len() {
        s_1 += f_1[i];
    }

    vec![s_0, s_1]
}

/// prover operation
pub fn prove(polynomial: &MultilinearPoly<Fq>) -> Proof {
    let mut transcript = Transcript::<Fq>::init();
    transcript.absorb(&to_bytes(&polynomial.evaluation));

        println!("Inside prove: polynomial size = {}", &polynomial.evaluation.len());
    assert!(!polynomial.evaluation.is_empty(), "Prove() received an empty polynomial");

    let claimed_sum: Fq = polynomial.evaluation.iter().sum();
    transcript.absorb(&to_bytes(&[claimed_sum]));

    let num_rounds = polynomial.evaluation.len().ilog2() as usize;
    let mut proof_polynomials = Vec::with_capacity(num_rounds as usize);
    let mut current_poly = polynomial.clone();

    for _ in 0..num_rounds {
        let proof_poly = partial_sum_proof(&current_poly.evaluation);

        transcript.absorb(&to_bytes(&proof_poly));

        proof_polynomials.push(proof_poly);

        let random_challenge = transcript.squeeze();

        current_poly = current_poly.partial_evaluate(0, &random_challenge);
    }

    Proof {
        proof_polynomials,
        claimed_sum,
    }

}

/// verifier operationpub 
pub fn verify(polynomial: &MultilinearPoly<Fq>, proof: Proof) -> bool {
    // Initializes the transcript
    let mut transcript = Transcript::<Fq>::init();
    transcript.absorb(&to_bytes(&polynomial.evaluation));
    transcript.absorb(&to_bytes(&[proof.claimed_sum]));

    let mut current_poly = polynomial.clone();
    let mut random_challenges = Vec::with_capacity(proof.proof_polynomials.len());
    let mut expected_sum = proof.claimed_sum;

    for poly in proof.proof_polynomials {
        let poly = MultilinearPoly::new(poly.to_vec());

        let mut sum = Fq::from(0);
        for eval in &poly.evaluation {
            sum += *eval;
        }

        if sum != expected_sum {
            return false;
        }

        transcript.absorb(&to_bytes(&poly.evaluation));

        let random_challenge = transcript.squeeze();

        expected_sum = poly.evaluation[0] + random_challenge * (poly.evaluation[1] - poly.evaluation[0]); 

        current_poly = current_poly.partial_evaluate(0, &random_challenge); 

        random_challenges.push(random_challenge);
    }

    let poly_eval_sum = polynomial.full_evaluation(random_challenges);

    expected_sum == poly_eval_sum
}


pub fn to_bytes(values: &[Fq]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|x| x.into_bigint().to_bytes_le())
        .collect()
}


pub fn generate_sumcheck_proof(
    claimed_sum: Fq,
    composed_polynomial: &SumPoly<Fq>,
    transcript: &mut Transcript<Fq>,
) -> GkrProof {
    let num_rounds = composed_polynomial.polys[0].evaluation[0].number_of_variables;
    let mut proof_polynomials = Vec::with_capacity(num_rounds as usize);
    let mut current_poly = composed_polynomial.clone();
    let mut random_challenges = Vec::new();

    for _ in 0..num_rounds {
        let proof_poly = get_round_partial_polynomial_proof_gkr(&current_poly);

        transcript.absorb(&fq_vec_to_bytes(&proof_poly.coefficient));

        proof_polynomials.push(proof_poly.coefficient);

        let random_challenge = transcript.squeeze();
        random_challenges.push(random_challenge);

        current_poly = current_poly.partial_evaluate(&random_challenge);
    }

    GkrProof {
        proof_polynomials,
        random_challenges,
        claimed_sum,
    }
}

// 
pub fn verify_sumcheck_proof(
    round_polys: Vec<UnivariatePoly<Fq>>,
    mut claimed_sum: Fq,
    transcript: &mut Transcript<Fq>,
) -> GkrVerify {
    let mut random_challenges = Vec::new();

    for round_poly in round_polys {
        let f_b_0 = round_poly.evaluate(Fq::from(0));
        let f_b_1 = round_poly.evaluate(Fq::from(1));

        if f_b_0 + f_b_1 != claimed_sum {
            return GkrVerify {
                verified: false,
                final_claimed_sum: Fq::from(0),
                random_challenges: vec![Fq::from(0)],
            };
        }

        transcript.absorb(&fq_vec_to_bytes(&round_poly.coefficient));

        let r_c = transcript.squeeze();

        random_challenges.push(r_c);

        claimed_sum = round_poly.evaluate(r_c); 
    }

    GkrVerify {
        verified: true,
        final_claimed_sum: claimed_sum,
        random_challenges,
    }
}


fn get_round_partial_polynomial_proof_gkr(composed_poly: &SumPoly<Fq>) -> UnivariatePoly<Fq> {
    let degree = composed_poly.get_degree();
    let mut poly_proof = Vec::with_capacity(degree + 1);

    for i in 0..=degree {
        let value = Fq::from(i as u64);
        let partial_poly = composed_poly.partial_evaluate(&value);
        let eval = partial_poly.reduce().iter().sum();
        poly_proof.push(eval);
    }

    let points: Vec<(Fq, Fq)> = poly_proof
        .iter()
        .enumerate()
        .map(|(i, y)| (Fq::from(i as u64), *y))
        .collect();

    let (xs, ys): (Vec<_>, Vec<_>) = points.into_iter().unzip();
    let poly = UnivariatePoly::interpolate(xs, ys);
    
    poly

}


fn get_round_partial_polynomial_proof(polynomial: &[Fq]) -> Vec<Fq> {
    let mid_point = polynomial.len() / 2;
    let (zeros, ones) = polynomial.split_at(mid_point);

    let poly_proof = vec![zeros.iter().sum(), ones.iter().sum()];

    poly_proof
}



#[cfg(test)]
mod test {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_valid_proving_and_verification() {
        // //  case 1
        let initial_polynomial = MultilinearPoly::new(vec![
            Fq::from(1),
            Fq::from(2),
            Fq::from(3),
            Fq::from(4),
            Fq::from(4),
            Fq::from(4),
            Fq::from(4),
            Fq::from(4),

        ]);
        let proof = prove(&initial_polynomial);
        let is_verified = verify(&initial_polynomial, proof);
        assert_eq!(is_verified, true);
     
    }

    #[test]
    fn test_invalid_proof_doesnt_verify() {
        let initial_polynomial =
            MultilinearPoly::new(vec![Fq::from(0), Fq::from(3), Fq::from(2), Fq::from(5)]);

        let tampered_claimed_sum = Fq::from(21); // correct sum is 10
        let proof = prove(&initial_polynomial);
        let false_proof = Proof {
            claimed_sum: tampered_claimed_sum,
            proof_polynomials: proof.proof_polynomials,
        };
        let is_verified = verify(&initial_polynomial, false_proof);
        assert_eq!(is_verified, false, "Tampered Claimed Sum Test Failed");

        let mut tampered_proof_polynomials = prove(&initial_polynomial).proof_polynomials;
        if let Some(first_poly) = tampered_proof_polynomials.first_mut() {
            if let Some(first_element) = first_poly.first_mut() {
                *first_element += Fq::from(1); 
            }
        }
        let false_proof = Proof {
            claimed_sum: prove(&initial_polynomial).claimed_sum,
            proof_polynomials: tampered_proof_polynomials,
        };

        let is_verified = verify(&initial_polynomial, false_proof);
        assert_eq!(is_verified, false);

    }

    #[test]
    fn test_intermediate_sum_check() {
        // This test focuses on checking intermediate sum at each round
        let initial_polynomial = MultilinearPoly::new(vec![Fq::from(1), Fq::from(2), Fq::from(3), Fq::from(4)]);
        let claimed_sum: Fq = initial_polynomial.evaluation.iter().sum();

        let mut transcript = Transcript::<Fq>::init();
        transcript.absorb(&to_bytes(&initial_polynomial.evaluation));
        transcript.absorb(&to_bytes(&[claimed_sum]));

        let num_rounds = initial_polynomial.evaluation.len().ilog2();
        let mut current_poly = initial_polynomial.clone();
        let mut expected_sum = claimed_sum;

        for _ in 0..num_rounds {
            let proof_poly = partial_sum_proof(&current_poly.evaluation);

            assert_eq!(proof_poly.iter().sum::<Fq>(), expected_sum, "Intermediate sum check failed!");

            transcript.absorb(&to_bytes(&proof_poly));
            let random_challenge = transcript.squeeze();

            expected_sum = proof_poly[0] + random_challenge * (proof_poly[1] - proof_poly[0]);
            current_poly = current_poly.partial_evaluate(0, &random_challenge);
        }
    }
}