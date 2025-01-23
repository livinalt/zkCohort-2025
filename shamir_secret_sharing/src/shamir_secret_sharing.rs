use crate::polynomials::Polynomial;
use std::vec::Vec;
// use ark_std::test_rng;

#[derive(Debug, Clone)]
pub struct Share {
    pub x: f64,
    pub y: f64,
}

pub struct SecretSharing {
    pub secret: f64,
    pub total_shares: usize,
    pub threshold: usize,
}

impl SecretSharing {
    pub fn new(secret: f64, total_shares: usize, threshold: usize) -> Self {
        SecretSharing {
            secret,
            total_shares,
            threshold,
        }
    }

    // Generate shares using polynomial interpolation
    pub fn generate_shares(&self) -> Vec<Share> {
        let mut coefficients = vec![self.secret];
        // let mut rng = test_rng();
        for _ in 1..self.threshold {
            let coefficient = rand::random::<f64>(); // Random coefficients
            coefficients.push(coefficient);
        }

        let polynomial = Polynomial::new(coefficients);

        let mut shares = Vec::new();
        for x in 1..=self.total_shares {
            let y = polynomial.evaluate(x as f64);
            shares.push(Share { x: x as f64, y });
        }

        shares
    }

    // Reconstructing the secret using Lagrange interpolation
    pub fn reconstruct_secret(&self, shares: &[Share]) -> f64 {
        let xs: Vec<f64> = shares.iter().map(|share| share.x).collect();
        let ys: Vec<f64> = shares.iter().map(|share| share.y).collect();

        let reconstructed_polynomial = Polynomial::interpolate(xs, ys);
        reconstructed_polynomial.evaluate(0.0) // Evaluating at x = 0 to get the secret
    }
}
