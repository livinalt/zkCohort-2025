use std::iter::{Product, Sum};
use std::ops::{Add, Mul};

#[derive(Debug, PartialEq, Clone)]
pub struct Polynomial {
    coefficient: Vec<f64>,
}

impl Polynomial {
    pub fn new(coefficient: Vec<f64>) -> Self {
        Polynomial { coefficient }
    }

    pub fn degree(&self) -> usize {
        for (i, coeff) in self.coefficient.iter().enumerate().rev() {
            if *coeff != 0.0 {
                return i;
            }
        }
        0
    }

    pub fn evaluate(&self, x: f64) -> f64 {
        let mut result = 0.0;
        let mut power = 1.0;

        for i in 0..self.coefficient.len() {
            result = result + self.coefficient[i] * power;
            power = power * x;
        }
        result
    }

    // Polynomial interpolation (Lagrange interpolation)
    pub fn interpolate(xs: Vec<f64>, ys: Vec<f64>) -> Self {
        let mut result = Polynomial::new(vec![0.0]);  // Start with a zero polynomial
        
        for i in 0..xs.len() {
            let mut basis_poly = Polynomial::new(vec![1.0]);  // l_i(x)

            for j in 0..xs.len() {
                if i != j {
                    let numerator = Polynomial::new(vec![(xs[i] - xs[j]), 1.0]);  // x - x_j
                    let denominator = xs[i] - xs[j]; // x_i - x_j

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

    fn scalar_mul(&self, scalar: &f64) -> Self {
        Polynomial {
            coefficient: self
                .coefficient
                .iter()
                .map(|coeff| coeff * scalar)
                .collect(),
        }
    }
}

impl Mul for &Polynomial {
    type Output = Polynomial;

    fn mul(self, rhs: Self) -> Self::Output {
        let new_degree = (self.degree() + rhs.degree()) as usize;
        let mut result = vec![0f64; new_degree + 1];

        for i in 0..self.coefficient.len() {
            for j in 0..rhs.coefficient.len() {
                result[i + j] = result[i + j] + self.coefficient[i] * rhs.coefficient[j];
            }
        }

        Polynomial {
            coefficient: result,
        }
    }
}

impl Add for &Polynomial {
    type Output = Polynomial;

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

        Polynomial::new(bigger.coefficient)
    }
}

impl Sum for Polynomial {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Polynomial::new(vec![0.0]);
        for poly in iter {
            result = &result + &poly;
        }
        result
    }
}

impl Product for Polynomial {
    fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut result = Polynomial::new(vec![1.0]);
        for poly in iter {
            result = &result * &poly;
        }
        result
    }
}
