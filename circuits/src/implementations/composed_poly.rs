use ark_ff::PrimeField;

use crate::implementations::multilinear_polynomial::MultilinearPoly;

// implementation of my composite poly
// composite fbc poly = ProductPoly[add(b,c), w_add(b,c)] 
// composite fbc poly = ProductPoly[add(b,c), w_add(b,c)] 

// [ ]  w(b) + w(c) → w_add(b, c)
// [ ]  w(b) * w(c) → w_mul(b, c)
// [ ]  ProductPoly[add(b,c), w_add(b,c)]
// [ ]  ProductPoly[mul(b, c), w_mul(b,c)]

// sumpoly = ProductPoly[add(b,c), w_add(b,c)] + ProductPoly[mul(b, c), w_mul(b,c)]


#[derive(Clone, Debug, PartialEq)]
pub struct ProductPoly<F: PrimeField> {
    pub evaluation: Vec<MultilinearPoly<F>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SumPoly<F: PrimeField> {
    pub polys: Vec<ProductPoly<F>>,
}

impl<F: PrimeField> ProductPoly<F> {
    pub fn init_poly(poly_evals: Vec<Vec<F>>) -> Self {
        let poly_length = poly_evals[0].len();

        if poly_evals.iter().any(|eval| eval.len() != poly_length) {
            panic!("all poly_evals must have same length");
        }

        let polys = poly_evals
            .iter()
            .map(|evaluation| MultilinearPoly::new(evaluation.to_vec()))
            .collect();

        Self { evaluation: polys }
    }

    fn evaluate(&self, values: Vec<F>) -> F {
        self.evaluation
            .iter()
            .map(|poly| poly.full_evaluation(values.clone()))
            .product()
    }

    fn partial_evaluate(&self, value: &F) -> Self {
        let partial_polys = self
            .evaluation
            .iter()
            .map(|poly| {
                let partial_res = poly.partial_evaluate(0, value);

                partial_res.evaluation
            })
            .collect();

        Self::init_poly(partial_polys)
    }

    fn reduce(&self) -> Vec<F> {
        (self.evaluation[0].clone() * self.evaluation[1].clone()).evaluation
    }

    fn get_degree(&self) -> usize {
        self.evaluation.len()
    }
}

impl<F: PrimeField> SumPoly<F> {
    pub fn new(polys: Vec<ProductPoly<F>>) -> Self {

        let poly_length = polys[0].get_degree();

        if polys.iter().any(|poly| poly.get_degree() != poly_length) {
            panic!("all product polys must have same degree");
        }

        Self { polys }
    }

    pub fn evaluate(&self, values: Vec<F>) -> F {

        self.polys
            .iter()
            .map(|poly| poly.evaluate(values.clone()))
            .sum()
    }

    pub fn partial_evaluate(&self, value: &F) -> Self {

        let partial_polys = self
            .polys
            .iter()
            .map(|product_poly| product_poly.partial_evaluate(value))
            .collect();

        Self::new(partial_polys)
    }

    pub fn reduce(&self) -> Vec<F> {

        let poly_a = &self.polys[0].reduce();
        let poly_b = &self.polys[1].reduce();

        let result = poly_a
            .iter()
            .zip(poly_b.iter())
            .map(|(a, b)| *a + *b)
            .collect();

        result
    }

    pub fn get_degree(&self) -> usize {
        self.polys[0].get_degree()
    }

}

#[cfg(test)]
mod test {
    use ark_bn254::Fq;

    use super::{ProductPoly, SumPoly};

    #[test]
    fn product_poly_evaluates_multiple_polys() {

        let poly_evals = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let product_polys = ProductPoly::init_poly(poly_evals);

        let values = vec![Fq::from(2), Fq::from(3)];

        let expected_evaluation = Fq::from(216);

        let result = product_polys.evaluate(values);

        assert_eq!(expected_evaluation, result);
    }

    #[test]
    fn product_poly_partially_evaluates_multiple_polys() {
        
        let poly_evals = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let product_polys = ProductPoly::init_poly(poly_evals);

        let value = Fq::from(2);

        let expected_evaluation = vec![
            vec![Fq::from(0), Fq::from(6)],
            vec![Fq::from(0), Fq::from(4)],
        ];

        let result = product_polys.partial_evaluate(&value);

        let result_polys: Vec<_> = result
            .evaluation
            .iter()
            .map(|poly| poly.evaluation.clone())
            .collect();

        assert_eq!(result_polys, expected_evaluation);
    }

    #[test]
    #[should_panic]
    fn product_poly_doesnt_allow_different_evaluation_size() {
        let poly_evals = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(4),
                Fq::from(0),
                Fq::from(0),
                Fq::from(0),
                Fq::from(4),
            ],
        ];

        let _ = ProductPoly::init_poly(poly_evals);
    }

    #[test]
    fn product_poly_gets_correct_degree() {}

    #[test]
    fn sum_poly_gets_correct_degree() {}

    #[test]
    fn sum_poly_evaluates_properly() {
        let poly_evals_1 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let poly_evals_2 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(4)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(5)],
        ];

        let product_poly_1 = ProductPoly::init_poly(poly_evals_1);
        let product_poly_2 = ProductPoly::init_poly(poly_evals_2);

        let sum_poly = SumPoly::new(vec![product_poly_1, product_poly_2]);

        let values = vec![Fq::from(2), Fq::from(3)];

        let expected_result = Fq::from(936);

        let result = sum_poly.evaluate(values);

        assert_eq!(expected_result, result);
    }

    #[test]
    fn sum_poly_partially_evaluates_properly() {
        let poly_evals_1 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(3)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(2)],
        ];

        let poly_evals_2 = vec![
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(4)],
            vec![Fq::from(0), Fq::from(0), Fq::from(0), Fq::from(5)],
        ];

        let product_poly_1 = ProductPoly::init_poly(poly_evals_1);
        let product_poly_2 = ProductPoly::init_poly(poly_evals_2);

        let value = Fq::from(2);

        let expected_evaluation_1 = vec![
            vec![Fq::from(0), Fq::from(6)],
            vec![Fq::from(0), Fq::from(4)],
        ];

        let expected_evaluation_2 = vec![
            vec![Fq::from(0), Fq::from(8)],
            vec![Fq::from(0), Fq::from(10)],
        ];

        let sum_poly = SumPoly::new(vec![product_poly_1, product_poly_2]);

        let result = sum_poly.partial_evaluate(&value);

        let result_polys: Vec<_> = result
            .polys
            .iter()
            .map(|product_poly| {
                product_poly
                    .evaluation
                    .iter()
                    .map(|poly| poly.evaluation.clone())
                    .collect::<Vec<_>>()
            })
            .collect();

        assert_eq!(
            vec![expected_evaluation_1, expected_evaluation_2],
            result_polys
        );
    }
}