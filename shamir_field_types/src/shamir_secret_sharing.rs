use ark_ff::PrimeField;
use ark_std::test_rng;

use crate::polynomials::Polynomial;
use std::vec::Vec;

#[derive(Debug, Clone)]
pub struct Share<F> {
    pub x: F,
    pub y: F,
}

pub struct SecretSharing<F> {
    pub secret: F,
    pub total_shares: u64,
    pub threshold: u64,    
}

impl<F: PrimeField> SecretSharing<F> {
    pub fn new(secret: F, total_shares: u64, threshold: u64) -> Self {
        SecretSharing {
            secret,
            total_shares,
            threshold,
        }
    }

    // Generate shares using polynomial interpolation
    pub fn generate_shares(&self) -> Vec<Share<F>> {
        let mut coefficients = vec![self.secret];
        let mut rng = test_rng();
        for _ in 1..self.threshold {
            let random_coefficient = F::rand(&mut rng);
            dbg!(&random_coefficient);

            coefficients.push(random_coefficient);
        }

        let polynomial = Polynomial::new(coefficients);

        let mut shares = Vec::new();
        for x in 1..=self.total_shares {
            let y = polynomial.evaluate(F::from(x)); 
            shares.push(Share { x: F::from(x), y });
        }

        shares
    }

    // Reconstructing the secret using Lagrange interpolation
    pub fn reconstruct_secret(&self, shares: &[Share<F>]) -> F {
        let xs: Vec<F> = shares.iter().map(|share| share.x).collect();
        let ys: Vec<F> = shares.iter().map(|share| share.y).collect();

        let reconstructed_polynomial = Polynomial::interpolate(xs, ys);
        reconstructed_polynomial.evaluate(F::zero()) // Evaluating at x = 0 to get the secret
    }
}