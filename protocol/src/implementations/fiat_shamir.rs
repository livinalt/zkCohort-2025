use ark_ff::{BigInteger, PrimeField};
use sha3::{digest::FixedOutputReset, Digest, Keccak256};

#[derive(Clone)]
struct Transcript<H: Digest> {
    hasher: H,
}

impl<H: Digest + FixedOutputReset> Transcript<H> {
    fn new() -> Self {
        Self {
            hasher: H::new(),
        }
    }

    fn absorb(&mut self, data: &[u8]) {
        Digest::update(&mut self.hasher, data);
    }

    fn squeeze<F: PrimeField>(&mut self) -> F {
        let result = self.hasher.finalize_reset();
        F::from_random_bytes(&result).unwrap()
    }
}

// Prover struc
struct Prover<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: Transcript<Keccak256>,
}

impl<F: PrimeField> Prover<F> {
    fn init(polynomial: Vec<F>, transcript: Transcript<Keccak256>) -> Self {
        Self {
            polynomial,
            transcript,
        }
    }

    fn generate_sum_and_univariate_poly(&self, evaluation: &[F]) -> (F, Vec<F>) {
        let claimed_sum = evaluation.iter().sum();
        let evaluation_len = evaluation.len() / 2;
        let mut univariate_poly = vec![];
        univariate_poly.push(evaluation.iter().take(evaluation_len).sum());
        univariate_poly.push(evaluation.iter().skip(evaluation_len).sum());
        (claimed_sum, univariate_poly)
    }

    fn generate_proof(&mut self, evaluation: Vec<F>) -> Proof<F> {
        let mut proof_array = vec![];
        let mut current_poly = evaluation.clone();
        let no_of_vars = (evaluation.len() as f64).log2() as usize;

        for _ in 0..no_of_vars {
            let (sum, univariate_poly) = self.generate_sum_and_univariate_poly(&current_poly);
            proof_array.push((sum, univariate_poly.clone()));

            // Absorb sum and univariate polynomial into the transcript
            self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
            self.transcript.absorb(
                &univariate_poly
                    .iter()
                    .map(|y| y.into_bigint().to_bytes_be())
                    .collect::<Vec<_>>()
                    .concat(),
            );

            // Generate a random challenge
            let random_challenge = self.transcript.squeeze();
            current_poly = self.evaluate_interpolate(current_poly.clone(), random_challenge);
        }

        Proof {
            univars_and_sums: proof_array,
        }
    }

    fn evaluate_interpolate(&self, poly: Vec<F>, point: F) -> Vec<F> {
        let mut result = vec![];
        for i in 0..poly.len() / 2 {
            result.push(poly[i] + point * (poly[i + poly.len() / 2] - poly[i]));
        }
        result
    }
}

// Proof struct
#[derive(Debug)]
struct Proof<F: PrimeField> {
    univars_and_sums: Vec<(F, Vec<F>)>,
}

// Verifier struct
struct Verifier<F: PrimeField> {
    polynomial: Vec<F>,
    transcript: Transcript<Keccak256>,
    proof: Proof<F>,
}

impl<F: PrimeField> Verifier<F> {
    fn init(polynomial: Vec<F>, transcript: Transcript<Keccak256>, proof: Proof<F>) -> Self {
        Self {
            polynomial,
            transcript,
            proof,
        }
    }

    fn verify(&mut self) -> bool {
        let proof = &self.proof.univars_and_sums;
        let mut random_challenge_array: Vec<F> = vec![];

        for i in 0..proof.len() {
            let (sum, univariate_poly) = &proof[i];
            let mut claimed_sum: F = *sum;
            let claim: F = univariate_poly.iter().sum();

            // Verify that the claimed sum matches the computed sum
            if claimed_sum != claim {
                return false;
            }

            // Absorb sum and univariate polynomial into the transcript
            self.transcript.absorb(sum.into_bigint().to_bytes_be().as_slice());
            self.transcript.absorb(
                &univariate_poly
                    .iter()
                    .map(|y| y.into_bigint().to_bytes_be())
                    .collect::<Vec<_>>()
                    .concat(),
            );

            // random challenge r
            let random_challenge = self.transcript.squeeze();
            random_challenge_array.push(random_challenge);

            // Evaluate the polynomial at the random challenge
            let current_poly = self.evaluate_interpolate(univariate_poly.clone(), random_challenge);
            claimed_sum = current_poly[0];

            // Verify the claimed sum === the next sum in the proof
            if i + 1 < proof.len() && claimed_sum != proof[i + 1].0 {
                return false;
            }

            // Oracle check Verify the polynomial evaluation === the claimed sum
            if i == proof.len() - 1 {
                let mut poly = self.polynomial.clone();
                for random_challenge in random_challenge_array.iter() {
                    poly = self.evaluate_interpolate(poly.clone(), *random_challenge);
                }
                if poly[0] != current_poly[0] {
                    return false;
                }
            }
        }
        true
    }

    fn evaluate_interpolate(&self, poly: Vec<F>, point: F) -> Vec<F> {
        let mut result = vec![];
        for i in 0..poly.len() / 2 {
            result.push(poly[i] + point * (poly[i + poly.len() / 2] - poly[i]));
        }
        result
    }
}

// Test case
#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::Field;
    use ark_bn254::Fr;

    #[test]
    fn test_prover_and_verifier() {
        let polynomial = vec![Fr::from(1), Fr::from(2), Fr::from(3)];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![Fr::from(6), Fr::from(11), Fr::from(22)];

        let proof = prover.generate_proof(evaluation);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(verifier.verify());
    }

    #[test]
    fn test_different_polynomial() {
        //different polynomial: f(x) = 4x^3 + 3x^2 + 2x + 1
        let polynomial = vec![Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4)];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![Fr::from(10), Fr::from(35), Fr::from(102), Fr::from(259)];

        let proof = prover.generate_proof(evaluation);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(verifier.verify());
    }

    #[test]
    fn test_empty_polynomial() {
        // test an empty polynomial
        let polynomial: Vec<Fr> = vec![];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![];

        let proof = prover.generate_proof(evaluation);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(verifier.verify());
    }

    #[test]
    fn test_invalid_proof() {

        let polynomial = vec![Fr::from(2), Fr::from(32), Fr::from(3)];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![Fr::from(6), Fr::from(11), Fr::from(22)];

        let mut proof = prover.generate_proof(evaluation);

        // Tamper with the proof to make it invalid
        proof.univars_and_sums[0].0 = Fr::from(0);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(!verifier.verify());
    }

    #[test]
    fn test_higher_degree_polynomial() {
        // test a higher degree polynomial: f(x) = x^4 + 2x^3 + 3x^2 + 4x + 5
        let polynomial = vec![Fr::from(5), Fr::from(4), Fr::from(3), Fr::from(2), Fr::from(1)];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![Fr::from(15), Fr::from(57), Fr::from(167), Fr::from(393), Fr::from(807)];

        let proof = prover.generate_proof(evaluation);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(verifier.verify());
    }

    #[test]
    fn test_single_point_polynomial() {
        // Define a polynomial with a single point: f(x) = 7
        let polynomial = vec![Fr::from(7)];
        let transcript = Transcript::<Keccak256>::new();
        let mut prover = Prover::init(polynomial.clone(), transcript.clone());

        let evaluation = vec![Fr::from(7)];

        let proof = prover.generate_proof(evaluation);

        let mut verifier = Verifier::init(polynomial, transcript, proof);

        assert!(verifier.verify());
    }
}