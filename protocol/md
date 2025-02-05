use std::{marker::PhantomData, vec};
use ark_ff::PrimeField;
use sha3::{digest::{Update, Digest}, Keccak256};

pub struct Transcript<F: PrimeField> {
    _field: PhantomData<F>,
    keccak: Keccak256,
}

impl<F: PrimeField> Transcript<F> {
    pub fn init() -> Self {
        Self { 
            _field: PhantomData, 
            keccak: Keccak256::default(),
        }
    }

    pub fn absorb(&mut self, data: &[u8]) {
        Digest::update(&mut self.keccak, data);
    }

    pub fn squeeze(&mut self) -> F {
        let hash_output: Vec<u8> = self.keccak.clone().finalize().to_vec();
        self.keccak = Keccak256::default(); // Reset the Keccak state
        F::from_be_bytes_mod_order(&hash_output)
    }
}

struct Prover<F: PrimeField, P: Fn(&[F]) -> F> {
    polynomial: P,
    _marker: PhantomData<F>,
    num_variables: usize,
}

impl<F: PrimeField, P: Fn(&[F]) -> F> Prover<F, P> {
    fn new(polynomial: P, num_variables: usize) -> Self {
        Prover {
            polynomial,
            _marker: PhantomData,
            num_variables,
        }
    }

    fn evaluate(&self, fixed_inputs: &[F], variable: usize) -> (F, F) {
        let mut args = fixed_inputs.to_vec();

        while args.len() < self.num_variables {
            args.push(F::zero()); 
        }

        let mut f_0 = args.clone();
        f_0[variable] = F::zero();
        let f_0 = (self.polynomial)(&f_0);

        let mut f_1 = args.clone();
        f_1[variable] = F::one();
        let f_1 = (self.polynomial)(&f_1);

        (f_0, f_1)
    }

    fn generate_proof(&self, claimed_sum: F) -> (Vec<F>, Vec<F>) {
        let mut transcript = Transcript::init();
        let mut fixed_inputs = Vec::new();
        let mut challenges = Vec::new();
        let mut expected_sum = claimed_sum;

        for variable in 0..self.num_variables {
            let (f_0, f_1) = self.evaluate(&fixed_inputs, variable);
            let sum = f_0 + f_1;

            if sum != expected_sum {
                panic!("Prover failed to generate proof: sum mismatch at variable {}", variable);
            }

            let mut bytes = vec![];
            bytes.extend_from_slice(&ark_ff::BigInteger::to_bytes_be(&sum.into_bigint()));
            transcript.absorb(&bytes);
            let random_challenge = transcript.squeeze();
            fixed_inputs.push(random_challenge);
            challenges.push(random_challenge);

            expected_sum = f_0 + (f_1 - f_0) * random_challenge;
        }

        (fixed_inputs, challenges)
    }
}

struct Verifier<F: PrimeField, P: Fn(&[F]) -> F> {
    polynomial: P,
    claimed_sum: F,
    num_variables: usize,
}

impl<F: PrimeField, P: Fn(&[F]) -> F> Verifier<F, P> {
    fn new(polynomial: P, claimed_sum: F, num_variables: usize) -> Self {
        Verifier {
            polynomial,
            claimed_sum,
            num_variables,
        }
    }

    fn verify(&self, proof: (Vec<F>, Vec<F>)) -> bool {
        let (fixed_inputs, challenges) = proof;
        let mut expected_sum = self.claimed_sum;

        for (variable, &challenge) in challenges.iter().enumerate() {
            let (f_0, f_1) = self.evaluate(&fixed_inputs, variable);
            let sum = f_0 + f_1;

            if sum != expected_sum {
                return false;
            }

            expected_sum = f_0 + (f_1 - f_0) * challenge;
        }

        true
    }

    fn evaluate(&self, fixed_inputs: &[F], variable: usize) -> (F, F) {
        let mut args = fixed_inputs.to_vec();

        while args.len() < self.num_variables {
            args.push(F::zero()); 
        }

        let mut f_0 = args.clone();
        f_0[variable] = F::zero();
        let f_0 = (self.polynomial)(&f_0);

        let mut f_1 = args.clone();
        f_1[variable] = F::one();
        let f_1 = (self.polynomial)(&f_1);

        (f_0, f_1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::{Fq, Fr};
    use ark_ff::One;

    #[test]
    fn test_transcript() {
        let mut transcript: Transcript<Fq> = Transcript::init();

        let data1 = b"Hello, ";
        let data2 = b"world!";
        
        transcript.absorb(data1);
        transcript.absorb(data2);

        let result = transcript.squeeze();

        let expected_hash = {
            let mut hasher = Keccak256::default();
            sha3::digest::Update::update(&mut hasher, data1);
            sha3::digest::Update::update(&mut hasher, data2);
            hasher.finalize().to_vec()
        };

        let expected_result = Fq::from_be_bytes_mod_order(&expected_hash);

        assert_eq!(result, expected_result, "The squeezed result does not match the expected hash.");
    }

    #[test]
    fn test_sum_check_valid_claim() {
        let polynomial = |vars: &[Fr]| -> Fr {
            let a = vars[0];
            let b = vars[1];
            let c = vars[2];
            Fr::from(2u64) * a * b + Fr::from(3u64) * b * c
        };

        let num_variables = 3;
        let prover = Prover::new(polynomial, num_variables);
        let claimed_sum = Fr::from(10u64);
        let proof = prover.generate_proof(claimed_sum);

        let verifier = Verifier::new(polynomial, claimed_sum, num_variables);
        assert!(
            verifier.verify(proof),
            "Sum-check protocol should succeed for valid claim"
        );
    }

    #[test]
    fn test_sum_check_invalid_claim() {
        let polynomial = |vars: &[Fr]| -> Fr {
            let a = vars[0];
            let b = vars[1];
            let c = vars[2];
            Fr::from(2u64) * a * b + Fr::from(3u64) * b * c
        };

        let num_variables = 3;
        let prover = Prover::new(polynomial, num_variables);
        let claimed_sum = Fr::from(10u64);
        let proof = prover.generate_proof(claimed_sum);

        let incorrect_claimed_sum = Fr::from(11u64);
        let verifier = Verifier::new(polynomial, incorrect_claimed_sum, num_variables);
        assert!(
            !verifier.verify(proof),
            "Sum-check protocol should fail for invalid claim"
        );
    }

    #[test]
    fn test_prover_evaluation() {
        let polynomial = |vars: &[Fr]| -> Fr {
            let a = vars[0];
            let b = vars[1];
            let c = vars[2];
            Fr::from(2u64) * a * b + Fr::from(3u64) * b * c
        };

        let fixed_inputs = vec![Fr::one(), Fr::one()];
        let num_variables = 3;
        let prover = Prover::new(polynomial, num_variables);
        let (f_0, f_1) = prover.evaluate(&fixed_inputs, 2);

        assert_eq!(f_0, Fr::from(2u64), "Prover evaluation at c = 0 is incorrect");
        assert_eq!(f_1, Fr::from(5u64), "Prover evaluation at c = 1 is incorrect");
    }
}