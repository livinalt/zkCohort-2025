use std::marker::PhantomData;
use ark_ff::{PrimeField, Zero};
use rand::Rng;
use ark_bn254::Fr;
use ark_std::test_rng;

///  SumCheck protocol implementation
pub struct SumCheck<F: PrimeField> {
    _field: PhantomData<F>,
}

/// Prover state
#[derive(Clone)]
pub struct ProverState<F: PrimeField> {
    evaluations: Vec<F>, // stores polynomial evaluations at all points
    rng: test_rng(),
    num_vars: usize,    // number of variables in the polynomial
}

/// Verifier state
pub struct VerifierState<F: PrimeField> {
    claimed_sum: F,
    current_sum: F,
    challenges: Vec<F>,
    poly_vars: usize,
}

impl<F: PrimeField> SumCheck<F> {
    pub fn initialize(evaluations: Vec<F>, num_vars: usize) -> (ProverState<F>, VerifierState<F>) {
        // Compute the claimed sum 
        let claimed_sum = evaluations.iter().copied().sum();

        let prover = ProverState {
            evaluations,
            num_vars,
            rng: test_rng(),
        };

        let verifier = VerifierState {
            claimed_sum,
            current_sum: F::zero(),
            challenges: Vec::new(),
            poly_vars: num_vars,
        };

        (prover, verifier)
    }

    pub fn prover_round(prover: &mut ProverState<F>, round: usize) -> Vec<F> {
        assert!(round < prover.num_vars, "Invalid round number");

        let step = 1 << (prover.num_vars - round - 1);
        let mut coefficients = vec![F::zero(); 2];

        for i in 0..(prover.evaluations.len() / 2) {
            let low_idx = i * 2 * step;
            let high_idx = low_idx + step;

            coefficients[0] += prover.evaluations[low_idx];
            coefficients[1] += prover.evaluations[high_idx];
        }

        coefficients
    }

    pub fn verifier_round(
        verifier: &mut VerifierState<F>,
        coefficients: &[F],
        rng: &mut impl Rng,
    ) -> Result<F, &'static str> {
        let sum = coefficients[0] + coefficients[1];
        if verifier.challenges.is_empty() {
            if sum != verifier.claimed_sum {
                return Err("Initial sum mismatch");
            }
        } else if sum != verifier.current_sum {
            return Err("Intermediate sum mismatch");
        }

        let challenge = F::rand(rng);
        verifier.challenges.push(challenge);

        verifier.current_sum = coefficients[0] + challenge * (coefficients[1] - coefficients[0]);

        Ok(challenge)
    }

    pub fn final_verification(
        verifier: &VerifierState<F>,
        evaluations: &[F],
    ) -> bool {
        let mut index = 0;
        for (i, &challenge) in verifier.challenges.iter().enumerate() {
            index |= ((challenge != F::zero()) as usize) << (verifier.poly_vars - i - 1);
        }

        evaluations[index] == verifier.current_sum
    }
}
