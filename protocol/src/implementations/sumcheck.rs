use ark_ff::PrimeField;
use rand::rngs::OsRng;
use std::marker::PhantomData;

// Prover's state
#[derive(Debug)]
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

        // Store and evaluate for both f_0, f_1
        let mut f_0 = args.clone();
        f_0[variable] = F::zero();
        let f_0 = (self.polynomial)(&f_0);

        let mut f_1 = args.clone();
        f_1[variable] = F::one();
        let f_1 = (self.polynomial)(&f_1);

        (f_0, f_1)
    }
}

// Verifier's implementation
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

    fn verify(&self, prover: &Prover<F, P>) -> bool {
        let mut rng = OsRng;
        let mut fixed_inputs = Vec::new();
        let mut expected_sum = self.claimed_sum;

        for variable in 0..self.num_variables {

            // Prover sends the univariate polynomial evaluated at bhc 0s and 1s
            let (f_0, f_1) = prover.evaluate(&fixed_inputs, variable);

            // ==> Verifier checks the sum
            let sum = f_0 + f_1;
            if sum != expected_sum {
                println!(
                    "Verifier rejected at variable {}: sum mismatch (expected {}, got {})",
                    variable, expected_sum, sum
                );
                return false;
            }

            // Verifier picjks and send a random challenge
            let random_challenge = F::rand(&mut rng);
            fixed_inputs.push(random_challenge);

            // Update the expected sum for the next round
            expected_sum = f_0 + (f_1 - f_0) * random_challenge;
        }

        // performing oracle check
        let final_value = (self.polynomial)(&fixed_inputs);
        if final_value != expected_sum {
            println!(
                "Verifier rejected final oracle check: expected {}, got {}",
                expected_sum, final_value
            );
            return false;
        }

        println!("Verifier accepted the proof!");
        true
    }
}

fn main() {
    println!("Sum-check protocol");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr;
    use ark_ff::One;

    #[test]
    fn test_sum_check_valid_claim() {
        // Define the polynomial: f(a, b, c) = 2ab + 3bc
        let polynomial = |vars: &[Fr]| -> Fr {
            let a = vars[0];
            let b = vars[1];
            let c = vars[2];
            Fr::from(2u64) * a * b + Fr::from(3u64) * b * c
        };

        let num_variables = 3;
        // Correct sum over the boolean hypercube
        let prover = Prover::new(polynomial, num_variables);
        let claimed_sum = Fr::from(10u64);
        let verifier = Verifier::new(polynomial, claimed_sum, num_variables);
    
        assert!(
            verifier.verify(&prover),
            "Sum-check protocol should succeed for valid claim"
        );
    }

    #[test]
    fn test_sum_check_invalid_claim() {
        // f(a, b, c) = 2ab + 3bc
        let polynomial = |vars: &[Fr]| -> Fr {
            let a = vars[0];
            let b = vars[1];
            let c = vars[2];
            Fr::from(2u64) * a * b + Fr::from(3u64) * b * c
        };

        // Incorrect claim (correct sum is 10)
        let claimed_sum = Fr::from(10u64);
        let num_variables = 3;

        let prover = Prover::new(polynomial, num_variables);
        let verifier = Verifier::new(polynomial, claimed_sum, num_variables);

        assert!(
            !verifier.verify(&prover),
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

        assert_eq!(f_0,Fr::from(2u64),
            "Prover evaluation at c = 0 is incorrect"
        );

        assert_eq!(
            f_1,
            Fr::from(5u64),
            "Prover evaluation at c = 1 is incorrect"
        );
    }
}