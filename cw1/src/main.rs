fn main() {
    // Represent the details of the polynomial 2x + 5
    let poly = Polynomial {
        coeff: vec![2.0, 5.0],
    };

    let x = 2.0;
    println!("Hello, Dense world!");
    println!("----------------------------------------------------");
    println!("The degree of the polynomial is: {}", degree(&poly));
    println!("----------------------------------------------------");
    println!("The Polynomial Evaluation is: {} at x = {}", evaluate(&poly, x), x);
    println!("----------------------------------------------------");

    // using this polynomial points (1, 2), (2, 3), (3, 5)
    let points = vec![(2.0, 9.0), (3.0, 11.0)];
    let interpolated_poly = interpolate(&points);

    println!("Interpolated Polynomial Coefficients: {:?}", interpolated_poly.coeff);
}

struct Polynomial {
    coeff: Vec<f64>, 
}

fn degree(poly: &Polynomial) -> u64 {
    let mut degree = 0;

    // To determine the degree we loop through the coeff vec
    for i in (0..poly.coeff.len()).rev() {
        if poly.coeff[i] != 0.0 {
            degree = i;
            break;
        }
    }
    degree as u64
}

// evaluating the polynomial 
fn evaluate(poly: &Polynomial, x: f64) -> f64 {
    let mut result = 0.0;
    let mut power_of_x = 1.0;

    for &coeff in &poly.coeff {
        result += coeff * power_of_x;
        power_of_x *= x;
    }

    result
}

// Interpolate a polynomial that passes through a given set of points using lagrange expression
fn interpolate(points: &Vec<(f64, f64)>) -> Polynomial {
    let n = points.len();
    let mut coeff = vec![0.0; n];

    for i in 0..n {
        let mut li_coeff = vec![1.0]; 

        for j in 0..n {
            if i != j {
                let x_i = points[i].0;
                let x_j = points[j].0;
                li_coeff = multiply_polynomials(&li_coeff, &[1.0, -x_j]); 
                let denominator = x_i - x_j;

                // Divide coefficients of L_i(x) by (x_i - x_j)
                for k in 0..li_coeff.len() {
                    li_coeff[k] /= denominator;
                }
            }
        }

        // Scale L_i(x) by y_i and add to the final polynomial
        for (k, &c) in li_coeff.iter().enumerate() {
            if coeff.len() <= k {
                coeff.push(0.0);
            }
            coeff[k] += c * points[i].1;
        }
    }

    Polynomial { coeff }
}

// Multiply two polynomials represented as coefficient vectors
fn multiply_polynomials(poly1: &[f64], poly2: &[f64]) -> Vec<f64> {
    let mut result = vec![0.0; poly1.len() + poly2.len() - 1];

    for (i, &c1) in poly1.iter().enumerate() {
        for (j, &c2) in poly2.iter().enumerate() {
            result[i + j] += c1 * c2;
        }
    }

    result
}
