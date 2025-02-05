use ark_ff::PrimeField;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial<F: PrimeField> {
    pub number_of_variables: usize,
    pub evaluated_points: Vec<F>,
}

impl<F: PrimeField> Polynomial<F> {
    pub fn new(evaluated_points: Vec<F>, number_of_variables: usize) -> Self {
        Polynomial { evaluated_points, number_of_variables }
    }

    
    pub fn evaluate(&mut self, values: Vec<F>) -> F {
        for i in 0..values.len() {
            *self = self.partial_evaluate((self.number_of_variables - 1, values[i]));
        }
        self.evaluated_points[0]
    }


    pub fn partial_evaluate(&mut self, (position, value): (usize, F)) -> Self {
        let length = self.evaluated_points.len();

    if length % 2 != 0 {
                panic!("The number of evaluated points must be a power of 2");
            }

        let mut new_evaluated_points = vec![F::zero(); (length / 2).try_into().unwrap()];

        let unique_pairs_evals = Self::get_unique_pairs_evals(self.evaluated_points.clone(), position);
        println!("evals of Unique Pairs: {:?}", unique_pairs_evals);

        for (i, (eval_1, eval_2)) in unique_pairs_evals.iter().enumerate() {
            new_evaluated_points[i] = *eval_1 + value * (*eval_2 - eval_1);
        }

        Polynomial::new(new_evaluated_points, self.number_of_variables - 1)
    }


    fn get_unique_pairs_evals(arr: Vec<F>, pos: usize) -> Vec<(F, F)> {
        let mask = 1 << pos; // Mask for the current bit position
        let mut evals = Vec::new(); // Vector to store the unique pairs
        
        for i in 0..arr.len() {
            let pair = i ^ mask; // Pair index

            // Only process unique pairs by avoiding duplicates)
            if i < pair {
                evals.push((arr[i], arr[pair]));
            }
        }

        evals
    }

}


#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use ark_bn254::{Fq, Fr};

    pub(crate) fn to_field(input: Vec<u64>) -> Vec<Fr> {
        input.iter().map(|v| Fr::from(*v)).collect()
    }


    #[test]
    fn test_evaluate(){
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.evaluate(vec![Fq::from(1), Fq::from(5), Fq::from(3)]);
        assert_eq!(result, Fq::from(55));
    }

    #[test]

    fn test_evaluate_2(){
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(1),
                Fq::from(0),
                Fq::from(2),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.evaluate(vec![Fq::from(1), Fq::from(5), Fq::from(3)]);
        assert_eq!(result, Fq::from(55));
    }
    
    #[test]
    fn test_partial_evaluate_polynomial_a_2v() {
        let mut poly = Polynomial::<Fq> {
            evaluated_points: vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)],
            number_of_variables: 2,
        };

        let partial_evaluated_poly = poly.partial_evaluate((1, Fq::from(5)));
        assert_eq!(
            partial_evaluated_poly.evaluated_points,
            vec![Fq::from(0), Fq::from(17)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_b_2v() {
        let mut poly = Polynomial::<Fq> {
            evaluated_points: vec![Fq::from(0), Fq::from(2), Fq::from(0), Fq::from(5)],
            number_of_variables: 2,
        };

        let partial_evaluated_poly = poly.partial_evaluate((0, Fq::from(3)));
        assert_eq!(
            partial_evaluated_poly.evaluated_points,
            vec![Fq::from(6), Fq::from(15)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_a_3v() {
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.partial_evaluate((2, Fq::from(1)));
        assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_a() {
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.partial_evaluate((1, Fq::from(5)));
        assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(15), Fq::from(10), Fq::from(25)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_b() {
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.partial_evaluate((0, Fq::from(3)));
        assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(9), Fq::from(0), Fq::from(11)]
        );
    }

    #[test]
    fn test_evaluate_polynomial_c() {
        let mut test_poly = Polynomial::new(
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(3),
                Fq::from(0),
                Fq::from(0),
                Fq::from(2),
                Fq::from(5),
            ],
            3,
        );
        let result = test_poly.evaluate(vec![Fq::from(1), Fq::from(5), Fq::from(3)]);
        assert_eq!(result, Fq::from(55));
    }

}

    fn main() {
    println!("Hello, Multilinear Polynomials");
    }