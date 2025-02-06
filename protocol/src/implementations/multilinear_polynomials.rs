use ark_ff::PrimeField;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Polynomial<F: PrimeField> {
    pub evaluated_points: Vec<F>,
}

impl<F: PrimeField> Polynomial<F> {
    pub fn new(evaluated_points: Vec<F>) -> Self {
        Polynomial { evaluated_points }
    }

    
    pub fn evaluate(&mut self, values:Vec<(usize, F)>) -> F {
        for (position,val ) in values{
            *self = self.partial_evaluate((position, val));
        }
        self.evaluated_points[0]
    }

    // let length = self.coefficients.len();
    //     if 2_i32.pow(pos as u32 + 1u32) > length as i32 {
    //         panic!(
    //             "The position is out of range for this polynomial with {} coefficients",
    //             self.coefficients.len()
    //         );
    //     }

    //     let mut new_coefficients = vec![F::zero(); (&length / 2).try_into().unwrap()];

    //     let unique_pairs_coefficients = Self::get_unique_pairs_coefficients(self.coefficients.clone(), pos);
    //     println!(
    //         "Coefficients of Unique Pairs: {:?}",
    //         unique_pairs_coefficients
    //     );

    //     for (i, (c_i, c_pair_index)) in unique_pairs_coefficients.iter().enumerate() {
    //         new_coefficients[i] = *c_i + val * (*c_pair_index - c_i);
    //     }

    //     MultilinearPoly::new(new_coefficients)




    pub fn partial_evaluate(&mut self, (position, val): (usize, F)) -> Self {
        let length = self.evaluated_points.len();

    if length % 2 != 0 {
                panic!("The number of evaluated points must be a power of 2");
            }

        let mut new_evaluated_points = vec![F::zero(); (length / 2).try_into().unwrap()];

        let unique_pairs_evals = Self::get_unique_pairs_evals(self.evaluated_points.clone(), position);
        println!("evals of Unique Pairs: {:?}", unique_pairs_evals);

        for (i, (eval_1, eval_2)) in unique_pairs_evals.iter().enumerate() {
            new_evaluated_points[i] = *eval_1 + val * (*eval_2 - eval_1);
        }

        Polynomial::new(new_evaluated_points)
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
    fn test_evaluate() {
        let mut test_poly = Polynomial::new(vec![
            Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3),
            Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5),
        ]);
    let result = test_poly.partial_evaluate((1, Fq::from(5)));
       assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(15), Fq::from(10), Fq::from(25)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_a() {
        let mut test_poly = Polynomial::new(vec![
            Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3),
            Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5),
        ]);
        let result = test_poly.partial_evaluate((1, Fq::from(5)));
        assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(15), Fq::from(10), Fq::from(25)]
        );
    }

    #[test]
    fn test_partial_evaluate_polynomial_b() {
        let mut test_poly = Polynomial::new(vec![
            Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3),
            Fq::from(0), Fq::from(0), Fq::from(2), Fq::from(5),
        ]);
        let result = test_poly.partial_evaluate((0, Fq::from(3)));
        assert_eq!(
            result.evaluated_points,
            vec![Fq::from(0), Fq::from(9), Fq::from(0), Fq::from(11)]
        );
    }
}
