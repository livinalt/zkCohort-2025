use crate::implementations::multilinear_polynomial::MultilinearPoly;
use crate::implementations::transcript::{Transcript};
use ark_ff::{BigInteger, PrimeField};
use ark_bn254::Fq;
use sha3::{Keccak256, Digest};

#[derive(Debug, Clone)]
pub struct Proof {
    claimed_sum: Fq,
    proof_polynomials: Vec<Vec<Fq>>,
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

        let random_challenge = transcript.squeeze();            // Generates random challenge r

        expected_sum = poly.evaluation[0] + random_challenge * (poly.evaluation[1] - poly.evaluation[0]);        // Update the expected sum for the next round

        current_poly = current_poly.partial_evaluate(0, &random_challenge);       // Partially evaluate the current polynomial

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