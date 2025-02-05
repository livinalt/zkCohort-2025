use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use std::result;

use ark_ff::{PrimeField, Zero, One};

#[derive(Debug, PartialEq, Clone)]
pub struct UnivariatePoly <F>  {
    coefficient: Vec<F>,
}

impl <F: PrimeField>UnivariatePoly<F> {
    pub fn new(coefficient: Vec<F>) -> Self {
        UnivariatePoly { coefficient }
    }

    pub fn degree(&self) -> usize {
        
        self.coefficient.len() - 1
        
        // for (i, coeff) in self.coefficient.iter().enumerate().rev() {
        //     if *coeff != 0 {
        //         return i;
        //     }
        // }
        // 0
    }

    pub fn evaluate(&self, x: F) -> F {
        // let mut result=0;
        // let mut power = 1;

        // for i in 0..self.coefficient.len() {
        //     result = result + self.coefficient[i] * power;
        //     power = power * x;
        // }
        // result

        self.coefficient.iter().rev().cloned().reduce(|acc, curr|acc * x + curr).unwrap()
    }

    // Polynomial interpolation (Lagrange interpolation)
    pub fn interpolate(xs: Vec<F>, ys: Vec<F>) -> Self {
        let mut result = UnivariatePoly::<F>::new(vec![F::zero()]);  // Start with a zero polynomial
        
        for i in 0..xs.len() {
            let mut basis_poly = UnivariatePoly::<F>::new(vec![F::one()]);  // l_i(x)

            for j in 0..xs.len() {
                if i != j {
                    let numerator = UnivariatePoly::<F>::new(vec![(- xs[j]), F::one()]);  // x - x_j
                    let denominator = (xs[i] - xs[j]).inverse().unwrap(); // x_i - x_j

                    basis_poly = &basis_poly * &numerator;
                    basis_poly = basis_poly.scalar_mul(&denominator);  // We now multiply by the denominator
                }
            }

            let scalar = ys[i];
            let term = basis_poly.scalar_mul(&scalar);
            result = &result + &term;
        }
        result
    }

    fn scalar_mul(&self, scalar: &F) -> Self {
        UnivariatePoly {
            coefficient: self
                .coefficient
                .iter()
                .map(|coeff| *coeff * *scalar)
                .collect(),
        }
    }
}

impl <F: PrimeField> Mul for &UnivariatePoly<F> {
    type Output = UnivariatePoly<F>;

    fn mul(self, rhs: Self) -> Self::Output {

        // mull for dense
        let new_degree = (self.degree() + rhs.degree()) as usize;
        let mut result = vec![F::zero(); new_degree + 1];

        for i in 0..self.coefficient.len() {
            for j in 0..rhs.coefficient.len() {
                result[i + j] += self.coefficient[i] * rhs.coefficient[j];
            }
        }

        UnivariatePoly {
            coefficient: result,
        }
    }
}

impl<F: PrimeField>Add for &UnivariatePoly<F> {
    type Output = UnivariatePoly<F>;

    fn add(self, rhs: Self) -> Self::Output {
        let (mut bigger, smaller) = if self.degree() < rhs.degree() {
            (rhs.clone(), self)
        } else {
            (self.clone(), rhs)
        };

        let _ = bigger
            .coefficient
            .iter_mut()
            .zip(smaller.coefficient.iter())
            .map(|(b_coeff, s_coeff)| *b_coeff = *b_coeff + *s_coeff)
            .collect::<()>(); 

        UnivariatePoly::new(bigger.coefficient)
    }
}

impl <F: PrimeField>Sum for UnivariatePoly<F> {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = UnivariatePoly::new(vec![F::zero()]);
        for poly in iter {
            result = &result + &poly;
        }
        result
    }
}

impl <F: PrimeField> Product for UnivariatePoly<F> {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = UnivariatePoly::new(vec![F::one()]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use ark_ff::{FftField, UniformRand};
    use ark_std::test_rng;
    use ark_std::Zero;
    use ark_bn254::Fq;
use ark_bn254::Fq as F;

    #[test]
    fn test_new_and_degree() {
        let mut rng = test_rng();
        let coeffs: Vec<Fq> = (0..5).map(|_| Fq::rand(&mut rng)).collect();
        let poly = UnivariatePoly::new(coeffs.clone());
        
        assert_eq!(poly.coefficient, coeffs);
        assert_eq!(poly.degree(), coeffs.len() - 1);
    }

    #[test]
    fn test_evaluate() {
        let mut rng = test_rng();
        let coeffs = vec![F::one(), F::one(), F::one()]; // represents x^2 + x + 1
        let poly = UnivariatePoly::new(coeffs);
        
        let x = F::one();
        let result = poly.evaluate(x);
        assert_eq!(result, F::one() + F::one() + F::one());
    }

    #[test]
    fn test_interpolation() {
        let mut rng = test_rng();
        let xs = vec![F::zero(), F::one(), F::from(2u64)];
        let ys = vec![F::one(), F::one(), F::one()];
        
        let poly = UnivariatePoly::interpolate(xs.clone(), ys.clone());
        
        // Verify that the polynomial passes through all points
        for (&x, &y) in xs.iter().zip(ys.iter()) {
            assert_eq!(poly.evaluate(x), y);
        }
    }

    #[test]
    fn test_scalar_mul() {
        let mut rng = test_rng();
        let coeffs = vec![F::one(), F::one(), F::one()];
        let poly = UnivariatePoly::new(coeffs.clone());
        let scalar = F::from(2u64);
        
        let scaled = poly.scalar_mul(&scalar);
        assert_eq!(
            scaled.coefficient,
            coeffs.into_iter().map(|c| c * scalar).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_add() {
        let mut rng = test_rng();
        let coeffs1 = vec![F::one(), F::one()];
        let coeffs2 = vec![F::one(), F::one(), F::one()];
        let poly1 = UnivariatePoly::new(coeffs1.clone());
        let poly2 = UnivariatePoly::new(coeffs2.clone());
        
        let sum = &poly1 + &poly2;
        let expected = vec![F::one() + F::one(), F::one() + F::one(), F::one()];
        assert_eq!(sum.coefficient, expected);
    }

    #[test]
    fn test_mul() {
        let mut rng = test_rng();
        let coeffs1 = vec![F::one(), F::one()]; // x + 1
        let coeffs2 = vec![F::one(), F::one()]; // x + 1
        let poly1 = UnivariatePoly::new(coeffs1.clone());
        let poly2 = UnivariatePoly::new(coeffs2.clone());
        
        let product = &poly1 * &poly2;
        let expected = vec![F::one(), F::one() + F::one(), F::one()]; // x^2 + 2x + 1
        assert_eq!(product.coefficient, expected);
    }

    #[test]
    fn test_sum() {
        let mut rng = test_rng();
        let poly1 = UnivariatePoly::new(vec![F::one(), F::one()]);
        let poly2 = UnivariatePoly::new(vec![F::one(), F::one(), F::one()]);
        
        let sum: UnivariatePoly<F> = vec![poly1, poly2].into_iter().sum();
        let expected = vec![F::one() + F::one(), F::one() + F::one(), F::one()];
        assert_eq!(sum.coefficient, expected);
    }


}