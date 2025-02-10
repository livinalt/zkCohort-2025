// use sha3::{Digest, Keccak256};
// use std::marker::PhantomData;

// use ark_ff::PrimeField;

// pub struct Transcript<K: HashTrait, F: PrimeField> {
//     _field: PhantomData<F>,
//     hash_function: K,
// }

// impl<K: HashTrait, F: PrimeField> Transcript<K, F> {
//     pub fn init(hash_function: K) -> Self {
//         Self {
//             _field: PhantomData,
//             hash_function,
//         }
//     }

//     pub fn absorb(&mut self, data: &[u8]) {
//         self.hash_function.append(data);
//     }

//     pub fn squeeze(&self) -> F {
//         let hash_output = self.hash_function.generate_hash();
//         F::from_be_bytes_mod_order(&hash_output)
//     }
// }

// pub trait HashTrait {
//     fn append(&mut self, data: &[u8]);
//     fn generate_hash(&self) -> Vec<u8>;
// }

// impl HashTrait for Keccak256 {
//     fn append(&mut self, data: &[u8]) {
//         self.update(data)
//     }

//     fn generate_hash(&self) -> Vec<u8> {
//         self.clone().finalize().to_vec()
//     }
// }

use ark_ff::PrimeField;
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

pub struct Transcript<F: PrimeField> {
    _field: PhantomData<F>,
    hasher: Keccak256,
}

impl<F: PrimeField> Transcript<F> {
    pub fn init() -> Self {
        Self {
            _field: PhantomData,
            hasher: Keccak256::new(),
        }
    }

    pub fn absorb(&mut self, preimage: &[u8]) {
        self.hasher.update(preimage)
    }

    pub fn squeeze(&mut self) -> F {
        let random_challenge = self.hasher.finalize_reset();

        self.absorb(&random_challenge);

        F::from_le_bytes_mod_order(&random_challenge)
    }
}

#[cfg(test)]
mod test {

    use super::Keccak256;
    use super::Transcript;
    use ark_bn254::Fq;
    use ark_ff::{BigInteger, PrimeField};
    use sha3::Digest;

    #[test]
    fn test_hash() {
        let mut transcript = Transcript::<Fq>::init();

        transcript.absorb(Fq::from(7).into_bigint().to_bytes_be().as_slice());
        transcript.absorb("girl".as_bytes());

        let challenge = transcript.squeeze();
        let challenge1 = transcript.squeeze();

        dbg!(challenge);
        dbg!(challenge1);
    }
}