use ark_ff::PrimeField;
use sha3::{Keccak256, Digest};
use std::marker::PhantomData;

/// Transcript for Fiat-Shamir transformation
struct Transcript<F: PrimeField> {
    hasher: Keccak256,
    _field: PhantomData<F>,
}

impl<F: PrimeField> Transcript<F> {
    /// Initialize a new transcript
    fn init() -> Self {
        Self {
            hasher: Keccak256::new(),
            _field: PhantomData,
        }
    }

    /// Absorb data into the transcript
    fn absorb(&mut self, data: &[u8]) {
        self.hasher.update(data);
    }

    /// Squeeze a field element from the transcript
    fn squeeze(&mut self) -> F {
        let hash_output = self.hasher.finalize_reset();
        F::from_be_bytes_mod_order(&hash_output)
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq; // Assuming you're using the Fq field from ark_bn254

    #[test]
    fn test_transcript() {
        // Create a new Transcript instance
        let mut transcript = Transcript::<Fq>::init();

        // Sample data to absorb into the transcript
        let data1 = b"Hello, ";
        let data2 = b"world!";

        // Absorb the first piece of data
        transcript.absorb(data1);

        // Absorb the second piece of data
        transcript.absorb(data2);

        // Squeeze the hash output from the transcript
        let result = transcript.squeeze();

        // Expected output calculation (for testing purposes)
        let expected_hash = {
            let mut hasher = Keccak256::new();
            hasher.update(data1);
            hasher.update(data2);
            hasher.finalize().to_vec()
        };

        // Convert expected hash to Fq type
        let expected_result = Fq::from_be_bytes_mod_order(&expected_hash);

        // Assert that the squeezed result matches the expected result
        assert_eq!(result, expected_result, "The squeezed result does not match the expected hash.");
    }
}