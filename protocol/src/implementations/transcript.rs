use std::{marker::PhantomData, vec};
use ark_ff::PrimeField;
use sha3::{digest::{Update, Digest}, Keccak256};
use ark_bn254::Fq; 

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

    pub fn squeeze(&self) -> F {
        let hash_output: Vec<u8> = self.keccak.clone().finalize().to_vec();
        F::from_be_bytes_mod_order(&hash_output)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;
    use sha3::Keccak256;

    #[test]
    fn test_transcript() {
        // Start by creating a new nstance of the Transcript
        let mut transcript: Transcript<Fq> = Transcript::init();

        // sample data to absorb into the transcript
        let data1 = b"Hello, ";
        let data2 = b"world!";
        
        // Absorb the first piece of data
        transcript.absorb(data1);
        
        // Absorb the second piece of data
        transcript.absorb(data2);

        // Squeeze the hash output from the transcript
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
}
