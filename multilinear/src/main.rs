use ark_ff::PrimeField;
use ark_ff::BigInteger;
use ark_bn254::Fr;


#[derive(Debug, Clone, PartialEq)]
pub struct MultilinearPoly<F: PrimeField> {
    evaluated_values: Vec<F>,
}

impl<F: PrimeField> MultilinearPoly<F> {
    pub fn new(evaluated_values: Vec<F>) -> Self {
        Self { evaluated_values }
    }

    // Evaluate the polynomial at a given point
    pub fn evaluate(&self, values: Vec<F>) -> F {
        let mut cloned_poly = self.evaluated_values.clone();
        let number_of_partial_evaluation = values.len();

        for i in 0..number_of_partial_evaluation {
            cloned_poly = self.partial_evaluation(cloned_poly, values[i]);
        }

        cloned_poly[0]
    }

    // Convert evaluated values to bytes
    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        for value in &self.evaluated_values {
            bytes.extend(value.into_bigint().to_bytes_be());
        }

        bytes
    }

    /// Generate a partial evaluation by fixing one variable
    fn partial_evaluation(&self, cloned_poly: Vec<F>, fixed_a: F) -> Vec<F> {
        let half_size = cloned_poly.len() / 2;
        let mut new_coefficients = vec![F::zero(); half_size];

        for i in 0..half_size {
            new_coefficients[i] =
                cloned_poly[i] * (F::one() - fixed_a) + cloned_poly[i + half_size] * fixed_a;
        }

        new_coefficients
    }

    // Reconstructs the polynomial
    pub fn interpolate(evaluations: &[F]) -> MultilinearPoly<F> {
        let n = evaluations.len();
        assert!(n.is_power_of_two(), "Number of evaluations must be a power of 2");

        let mut coefficients = evaluations.to_vec();
        let mut step = 1;

        while step < n {
            for i in 0..(n / (2 * step)) {
                for j in 0..step {
                    let idx1 = i * 2 * step + j;
                    let idx2 = idx1 + step;

                    coefficients[idx1] =
                        (coefficients[idx1] + coefficients[idx2]) * F::from(2u64).inverse().unwrap();
                    coefficients[idx2] =
                        (coefficients[idx1] - coefficients[idx2]) * F::from(2u64).inverse().unwrap();
                }
            }
            step *= 2;
        }

        MultilinearPoly::new(coefficients)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fq;

    #[test]
    fn test_evaluate() {
        let poly = MultilinearPoly::new(vec![
            Fq::from(1u64),
            Fq::from(2u64),
            Fq::from(3u64),
            Fq::from(4u64),
        ]);

        // Evaluate at (0, 0)
        assert_eq!(poly.evaluate(vec![Fq::from(0u64), Fq::from(0u64)]), Fq::from(1u64));

        // Evaluate at (1, 0)
        assert_eq!(poly.evaluate(vec![Fq::from(1u64), Fq::from(0u64)]), Fq::from(3u64));

        // Evaluate at (0, 1)
        assert_eq!(poly.evaluate(vec![Fq::from(0u64), Fq::from(1u64)]), Fq::from(2u64));

        // Evaluate at (1, 1)
        assert_eq!(poly.evaluate(vec![Fq::from(1u64), Fq::from(1u64)]), Fq::from(4u64));
    }

    #[test]
    fn test_convert_to_bytes() {
        let poly = MultilinearPoly::new(vec![
            Fq::from(1u64),
            Fq::from(2u64),
            Fq::from(3u64),
            Fq::from(4u64),
        ]);

        let bytes = poly.convert_to_bytes();
        assert!(!bytes.is_empty());
    }


     #[test]

    fn test_evaluate_three_variables() {
        //for a three-variable polynomial with 8 evaluated values
        let poly = MultilinearPoly::new(vec![
            Fq::from(1u64),  // f(0, 0, 0)
            Fq::from(2u64),  // f(0, 0, 1)
            Fq::from(3u64),  // f(0, 1, 0)
            Fq::from(4u64),  // f(0, 1, 1)
            Fq::from(5u64),  // f(1, 0, 0)
            Fq::from(6u64),  // f(1, 0, 1)
            Fq::from(7u64),  // f(1, 1, 0)
            Fq::from(8u64),  // f(1, 1, 1)
        ]);

        // Evaluate at (0, 0, 0)
        assert_eq!(
            poly.evaluate(vec![Fq::from(0u64), Fq::from(0u64), Fq::from(0u64)]),
            Fq::from(1u64)
        );

        // Evaluate at (1, 0, 0)
        assert_eq!(
            poly.evaluate(vec![Fq::from(1u64), Fq::from(0u64), Fq::from(0u64)]),
            Fq::from(5u64)
        );

        // Evaluate at (0, 1, 0)
        assert_eq!(
            poly.evaluate(vec![Fq::from(0u64), Fq::from(1u64), Fq::from(0u64)]),
            Fq::from(3u64)
        );

        // Evaluate at (0, 0, 1)
        assert_eq!(
            poly.evaluate(vec![Fq::from(0u64), Fq::from(0u64), Fq::from(1u64)]),
            Fq::from(2u64)
        );

        // Evaluate at (1, 1, 1)
        assert_eq!(
            poly.evaluate(vec![Fq::from(1u64), Fq::from(1u64), Fq::from(1u64)]),
            Fq::from(8u64)
        );
    }


}
