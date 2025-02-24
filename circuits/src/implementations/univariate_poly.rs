use std::iter::{Product, Sum};
use std::ops::{Add, Mul};
use std::result;

use ark_bn254::Fq;
use ark_ff::{PrimeField, Zero};

use super::composed_poly::SumPoly;

#[derive(Debug, PartialEq, Clone)]
pub struct UnivariatePoly <F>  {
    pub coefficient: Vec<F>,
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

    // Polynomial interpolation (Lagrange interpolation)
    fn get_round_partial_polynomial_proof_gkr(composed_poly: &SumPoly<Fq>) -> UnivariatePoly<Fq> {
        let degree = composed_poly.get_degree();
        let mut poly_proof = Vec::with_capacity(degree + 1);

        for i in 0..=degree {
            let value = Fq::from(i as u64);
            let partial_poly = composed_poly.partial_evaluate(&value);
            let eval = partial_poly.reduce().iter().sum();
            poly_proof.push(eval);
        }

        let xs: Vec<Fq> = (0..=degree).map(|i| Fq::from(i as u64)).collect();
        let ys: Vec<Fq> = poly_proof.clone();  // Use y-values directly

        UnivariatePoly::interpolate(xs, ys)
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
        let mut result: UnivariatePoly<F> = UnivariatePoly::new(vec![F::zero()]);
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
