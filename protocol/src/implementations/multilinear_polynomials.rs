use ark_ff::{PrimeField, BigInteger};
use std::marker::PhantomData;
    use ark_bn254::Fq;

#[derive(Debug, Clone)]
pub struct Polynomial<F: PrimeField> {
    evaluated_points: Vec<F>,
    number_of_variables: i32,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> Polynomial<F> {
    pub fn init_poly(evaluated_points: Vec<F>, number_of_variables: i32) -> Self {
        Polynomial {
            evaluated_points,
            number_of_variables,
            _marker: PhantomData,
        }
    }

    pub fn evaluation(&self, challenges: &[F]) -> F {
        let num_vars = self.number_of_variables as usize;
        if challenges.len() != num_vars {
            panic!("Number of challenges must equal number of variables");
        }
        let mut sum = F::zero();
        for i in 0..self.evaluated_points.len() {
            let mut term = self.evaluated_points[i];
            for j in 0..num_vars {
                let bit = (i >> j) & 1;
                term *= if bit == 0 { challenges[j] } else { F::one() - challenges[j] };
            }
            sum += term;
        }
        sum
    }

    pub fn partial_evaluation(&self, var_index: usize) -> Result<Self, ()> {
        let num_vars = self.number_of_variables as usize;
        if var_index >= num_vars {
            return Err(());
        }

        let step = 2usize.pow((num_vars - var_index - 1) as u32);
        let mut new_evaluated_points = Vec::new();
        
        for i in (0..self.evaluated_points.len()).step_by(step * 2) {
            for j in 0..step {
                let left = self.evaluated_points[i + j];
                let right = self.evaluated_points[i + j + step];
                new_evaluated_points.push(left);
                new_evaluated_points.push(right);
            }
        }

        Ok(Polynomial::from_evaluated_points(
            new_evaluated_points,
            (num_vars - 1) as i32
        ))
    }

    pub fn evaluate_manually<G: PrimeField>(evaluated_points: &Vec<G>, challenges: &Vec<G>) -> G {
        let num_vars = challenges.len();
        let mut sum = G::zero();
        for i in 0..evaluated_points.len() {
            let mut term = evaluated_points[i];
            for j in 0..num_vars {
                let bit = (i >> j) & 1;
                term *= if bit == 0 { challenges[j] } else { G::one() - challenges[j] };
            }
            sum += term;
        }
        sum
    }

    pub fn get_evaluated_points(&self) -> &Vec<F> {
        &self.evaluated_points
    }

    pub fn number_of_variables(&self) -> usize {
        self.number_of_variables as usize
    }

    pub fn from_evaluated_points(evaluated_points: Vec<F>, num_vars: i32) -> Self {
        Polynomial {
            evaluated_points,
            number_of_variables: num_vars,
            _marker: PhantomData,
        }
    }
}

#[cfg(test)]
mod test {
    use ark_bn254::Fr;
    use super::{Polynomial};

    // fn test_manual_evaluation() {
    //     let evaluated_points: Vec<Fr> = vec![
    //         Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4),
    //         Fr::from(5), Fr::from(6), Fr::from(7), Fr::from(8),
    //     ];
    //     let num_vars = 3;
    //     let poly = Polynomial::init_poly(evaluated_points, num_vars);
    //     let challenges = vec![Fr::from(10), Fr::from(20), Fr::from(30)];
    //     let expected_evaluation = evaluate_manually(&evaluated_points, &challenges);
    //     let actual_evaluation = poly.evaluation(&challenges);
    //     assert_eq!(actual_evaluation, expected_evaluation, "Evaluation failed (3 vars)");
    // }

    #[test]
fn test_evaluation() {
    let evaluated_points: Vec<Fr> = vec![
        Fr::from(1), Fr::from(2), Fr::from(3), Fr::from(4),
        Fr::from(5), Fr::from(6), Fr::from(7), Fr::from(8),
    ];
    let num_vars = 3;
    let poly = Polynomial::init_poly(evaluated_points.clone(), num_vars);
    let challenges = vec![Fr::from(10), Fr::from(20), Fr::from(30)];
    let expected_evaluation = Polynomial::<Fr>::evaluate_manually(&evaluated_points, &challenges);
    let actual_evaluation = poly.evaluation(&challenges);
    assert_eq!(actual_evaluation, expected_evaluation, "Evaluation failed (3 vars)");
}

    #[test]
fn test_partial_evaluation() {
    let evaluated_points: Vec<Fr> = vec![Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(3),
        Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)];
    let number_of_variables: i32 = 3;
    let poly = Polynomial::<Fr>::init_poly(evaluated_points.clone(), number_of_variables);
    
    // Test each variable index separately
    for var_index in 0..number_of_variables {
        let mut current_poly = poly.clone(); // Start fresh for each iteration
        let result = Polynomial::<Fr>::partial_evaluation(&current_poly, var_index as usize)
            .expect("Partial evaluation should succeed");
        let current_points = result.get_evaluated_points();
        assert_eq!(current_points[0], evaluated_points[0]);
    }
    
    // Test out of bounds
    let result = Polynomial::<Fr>::partial_evaluation(&poly, number_of_variables as usize);
    // println!("{:?}", result);
    assert!(result.is_err());
}

    #[test]
    fn test_from_evaluated_points() {
        let evaluated_points: Vec<Fr> = vec![Fr::from(0), Fr::from(0), Fr::from(1), Fr::from(3),
                                            Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)];
        let number_of_variables: i32 = 3;
        let poly = Polynomial::<Fr>::from_evaluated_points(evaluated_points.clone(), number_of_variables);
        assert_eq!(poly.get_evaluated_points(), &evaluated_points);
        assert_eq!(poly.number_of_variables(), number_of_variables as usize);
    }
}