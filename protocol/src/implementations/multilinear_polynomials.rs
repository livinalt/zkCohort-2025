use ark_ff::{PrimeField, Fp256, BigInteger256};
use std::marker::PhantomData;
use ark_bn254::Fr;



// evaluated form: the values are from the BHC [0,0,0,3,0,0,2,5]
// a. define the struct
// b. insert the values of the variables
// c. solve the polynomial


struct Polynomial <F:PrimeField>{
    evaluated_points: Vec<F>,
    number_of_variables: i32,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> Polynomial<F> {

    
    pub fn init_poly(evaluated_points: Vec<F>, number_of_variables:i32) -> Polynomial<F> {
        let mut new_poly = Polynomial::<F> { evaluated_points: Vec::new(), number_of_variables, _marker: PhantomData };
        new_poly.evaluated_points = evaluated_points;
        new_poly
    }


    pub fn evaluation(new_poly: Polynomial<F>) -> F {
    let mut evaluation_sum = F::from(0u32);
    for i in 0..new_poly.evaluated_points.len(){
        evaluation_sum += new_poly.evaluated_points[i];
        }
        evaluation_sum
    }


    pub fn partial_evaluation(new_poly: &Polynomial<F>) -> Result<F, &'static str> {
        let mut evaluated_points = new_poly.evaluated_points.clone();
        let num_vars = new_poly.number_of_variables as usize;


        // Process each bit
        for bit in 0..num_vars {
            let step = 2_usize.pow((num_vars - bit - 1) as u32);
            let mut new_evaluated_points = Vec::new();

            // Pair and interpolate
            for i in (0..evaluated_points.len()).step_by(step * 2) {
                for j in 0..step {
                    let left = evaluated_points[i + j];
                    let right = evaluated_points[i + j + step];
                    let interpolated_value = left  + right;
                    new_evaluated_points.push(interpolated_value);
                }
            }

            // Update evaluated_points for the next iteration
            evaluated_points = new_evaluated_points;
        }

        // Return the final result
        Ok(evaluated_points[0])
}

}




#[cfg(test)]
mod test{
    use ark_bn254::Fr;
    use super::Polynomial;

#[test]
fn test_evaluation() {
    let evaluated_points: Vec<Fr> = vec![Fr::from(0), Fr::from(0), Fr::from(1), Fr::from(3), Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)];
    let number_of_variables: i32 = 3;

    let new_poly = Polynomial::<Fr>::init_poly(evaluated_points, number_of_variables);
    let evaluation_sum = Polynomial::evaluation(new_poly);

    println!("The evaluation sum is: {}", evaluation_sum);
    assert_eq!(evaluation_sum, Fr::from(11u32)); 
}

    #[test]
fn test_partial_evaluation() {
    let evaluated_points: Vec<Fr> = vec![Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(3), Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5), Fr::from(0), Fr::from(0), Fr::from(1), Fr::from(3), Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)];
    let number_of_variables: i32 = 4;

    let new_poly = Polynomial::<Fr>::init_poly(evaluated_points, number_of_variables);
    let result = Polynomial::<Fr>::partial_evaluation(&new_poly).unwrap();
    println!("{}", result);

    let expected_result = Fr::from(21);
    assert_eq!(result, expected_result, "Partial evaluation result is incorrect");
}

}

fn main() {
    println!("Hello, multilinear polynomial!");
}