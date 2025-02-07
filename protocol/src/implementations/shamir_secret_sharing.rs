use ark_ff::PrimeField;
use ark_std::test_rng;
use crate::implementations::polynomials::UnivariatePoly;
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
        
        let polynomial = UnivariatePoly::new(coefficients);
        
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

        let reconstructed_polynomial = UnivariatePoly::interpolate(xs, ys);
        reconstructed_polynomial.evaluate(F::zero()) // Evaluating at x = 0 to get the secret
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_std::test_rng;
    use ark_std::UniformRand;
    use ark_std::Zero;
    use ark_std::One;
    use ark_bn254::Fq;
    
    #[test]
    fn test_secret_sharing_basic() {
        let mut rng = test_rng();
        let secret: Fq = Fq::rand(&mut rng);
        let total_shares = 5;
        let threshold = 3;
        
        let sharing = SecretSharing::new(secret, total_shares, threshold);
        let shares = sharing.generate_shares();
        
        assert_eq!(shares.len() as u64, total_shares);
        
        for (i, share) in shares.iter().enumerate() {
            assert_eq!(share.x, Fq::from(i as u64 + 1));
        }
        
        let reconstructed = sharing.reconstruct_secret(&shares);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_secret_sharing_threshold() {
        let mut rng = test_rng();
        let secret: Fq = Fq::rand(&mut rng);
        let total_shares = 5;
        let threshold = 3;

        let sharing = SecretSharing::new(secret, total_shares, threshold);
        let shares = sharing.generate_shares();
        
        let reconstructed = sharing.reconstruct_secret(&shares[0..threshold as usize]);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_secret_sharing_below_threshold() {
        let mut rng = test_rng();
        let secret: Fq = Fq::rand(&mut rng);
        let total_shares = 5;
        let threshold = 3;

        let sharing = SecretSharing::new(secret, total_shares, threshold);
        let shares = sharing.generate_shares();
        
        let reconstructed = sharing.reconstruct_secret(&shares[0..(threshold - 1) as usize]);
        assert_ne!(reconstructed, secret);
    }
    

    #[test]
    fn test_secret_sharing_zero_secret() {
        let secret = Fq::zero();
        let total_shares = 5;
        let threshold = 3;
        
        let sharing = SecretSharing::new(secret, total_shares, threshold);
        let shares = sharing.generate_shares();
        
        let reconstructed = sharing.reconstruct_secret(&shares);
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_secret_sharing_one_secret() {
        let secret = Fq::one();
        let total_shares = 5;
        let threshold = 3;
        
        let sharing = SecretSharing::new(secret, total_shares, threshold);
        let shares = sharing.generate_shares();
        
        let reconstructed = sharing.reconstruct_secret(&shares);
        assert_eq!(reconstructed, secret);
    }
}


fn main(){
    println!("Hello, Shamir Secret Sharing")
}