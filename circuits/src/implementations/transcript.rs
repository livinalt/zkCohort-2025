use ark_bn254::Fq;
use ark_ff::{BigInteger, PrimeField};
use sha3::{Digest, Keccak256};
use std::marker::PhantomData;

#[derive(Clone)]
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

pub fn fq_vec_to_bytes(values: &[Fq]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|x| x.into_bigint().to_bytes_le())
        .collect()
}

#[cfg(test)]
mod test {

    use super::Transcript;
    use ark_bn254::Fq;
    use ark_ff::{BigInteger, PrimeField};

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