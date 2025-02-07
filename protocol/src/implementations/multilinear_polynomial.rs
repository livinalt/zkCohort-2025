use ark_ff::PrimeField;
use std::ops::Add;

#[derive(Clone)]
pub struct MultilinearPoly<F: PrimeField> {
    pub evaluation: Vec<F>,
    pub number_of_variables: usize,
}

impl<F: PrimeField> MultilinearPoly<F> {
    pub fn new(new_evaluations: Vec<F>) -> Self {
        let number_of_variables: usize = new_evaluations.len().ilog2() as usize;

        if new_evaluations.len() != 1 << number_of_variables {
            panic!("Invalid evaluations sets");
        }

        Self {
            evaluation: new_evaluations,
            number_of_variables,
        }
    }

    fn pair_points(bit: usize, number_of_variables: usize) -> Vec<(usize, usize)> {

        let mut result = vec![];
        let target_hc = number_of_variables - 1;

        for val in 0..(1 << target_hc) {
            let inverted_index = number_of_variables - bit - 1;
            let insert_zero = insert_bit(val, inverted_index);
            let insert_one = insert_zero | (1 << inverted_index);
            result.push((insert_zero, insert_one));
        }
        result
    }

        // empoying the equation of a straight line for partial evaluation
        // f(x) = y_0 + r(y_1 - y_0)
    pub fn partial_evaluate(&self, bit: usize, value: &F) -> Self {
        let mut result = vec![];

        for (y_0, y_1) in
            MultilinearPoly::<F>::pair_points(bit, self.number_of_variables).into_iter()
        {
            let y_0 = self.evaluation[y_0];
            let y_1 = self.evaluation[y_1];

            result.push(y_0 + *value * (y_1 - y_0));
        }

        Self::new(result)
    }

    pub fn full_evaluation(&self, values: Vec<F>) -> F {
        if values.len() != self.number_of_variables {
            panic!("Invalid number of values");
        }

        let mut result = self.clone();

        for value in values.iter() {
            result = result.partial_evaluate(0, value);
        }

        result.evaluation[0]
    }
}

impl<F: PrimeField> Add for MultilinearPoly<F> {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        let mut result = vec![F::zero(); self.evaluation.len().max(other.evaluation.len())];

        let len = self.evaluation.len();

        for i in 0..len {
            result[i] += self.evaluation[i];
        }

        let len = other.evaluation.len();

        for i in 0..len {
            result[i] += other.evaluation[i];
        }

        MultilinearPoly::new(result)
    }
}

fn insert_bit(value: usize, bit: usize) -> usize {
    let high = value >> bit;
    let mask = (1 << bit) - 1;
    let low = value & mask;

    high << (bit + 1) | low
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr as F; 
    use ark_ff::{One, Zero};

    #[test]
    fn test_new() {
        let evaluations = vec![F::zero(), F::one(), F::one(), F::zero()];
        let poly = MultilinearPoly::new(evaluations.clone());

        assert_eq!(poly.evaluation, evaluations);
        assert_eq!(poly.number_of_variables, 2);
    }

    #[test]
    #[should_panic]
    fn test_new_invalid_evaluations() {
        let _ = MultilinearPoly::<F>::new(vec![F::zero(), F::one()]);
    }

    #[test]
    fn test_partial_evaluate() {
        let evaluations = vec![F::zero(), F::one(), F::one(), F::zero()];
        let poly = MultilinearPoly::new(evaluations);

        let value = F::from(2u64); 
        let new_poly = poly.partial_evaluate(0, &value);

        assert_eq!(new_poly.evaluation.len(), 2);
    }

    #[test]
    fn test_full_evaluation() {
        let evaluations = vec![F::zero(), F::one(), F::one(), F::zero()];
        let poly = MultilinearPoly::new(evaluations);

        let values = vec![F::from(2u64), F::from(3u64)];
        let result = poly.full_evaluation(values);

        assert!(result != F::zero());
    }

    #[test]
    fn test_addition() {
        let poly1 = MultilinearPoly::new(vec![F::one(), F::zero(), F::one(), F::zero()]);
        let poly2 = MultilinearPoly::new(vec![F::zero(), F::one(), F::zero(), F::one()]);

        let sum = poly1.clone() + poly2.clone();

        assert_eq!(
            sum.evaluation,
            vec![F::one(), F::one(), F::one(), F::one()]
        );
    }
}
