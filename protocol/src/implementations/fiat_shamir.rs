// use crate::implementations::multilinear_polynomial::Polynomial;
// use crate::implementations::transcript::{Transcript, HashTrait};
// use ark_ff::{BigInteger, PrimeField};
// use sha3::Keccak256;

// struct Proof<F: PrimeField> {
//     claimed_sum: F,
//     round_polys: Vec<[F; 2]>,
// }

// fn prove<F: PrimeField>(mut poly: Polynomial<F>, claimed_sum: F) -> Proof<F> {
//     let mut round_polys: Vec<[F; 2]> = vec![];
//     let mut transcript = Transcript::<Keccak256, F>::init(Keccak256::default());
    
//     // Absorb initial polynomial evaluations and claimed sum.
//     transcript.absorb(
//     poly.evaluated_points
//             .iter()
//             .flat_map(|f| f.into_bigint().to_bytes_be())
//             .collect::<Vec<_>>()
//             .as_slice(),
//     );

//     transcript.absorb(
//         claimed_sum.into_bigint().to_bytes_be().as_slice()
//     );

    
//     for _ in 0..poly.number_of_variables {
//         // For branch checking: derive the sum of the evaluations when fixing the last variable to 0 and 1.
//         let poly_left = poly.partial_evaluate((0, F::zero()));
//             // .expect("Failed to partially evaluate polynomial for left branch");
//         let poly_right = poly.partial_evaluate((1, F::one()));
//             // .expect("Failed to partially evaluate polynomial for right branch");

//         let round_poly: [F; 2] = [

//             // LHS == f(0)
//             poly.partial_evaluate((poly.number_of_variables - 1, F::zero()))
//                 .evaluated_points
//                 .iter()
//                 .sum(),

//                 // RHS ==> f(1)
//             poly.partial_evaluate((poly.number_of_variables - 1, F::one())).evaluated_points.iter().cloned().sum(),
//         ];
//         transcript.absorb(
//             round_poly.iter()
//                 .flat_map(|f| f.into_bigint().to_bytes_be())
//                 .collect::<Vec<_>>()
//                 .as_slice()
//         );
//         round_polys.push(round_poly);
        
//         // Squeeze a challenge from the transcript.
//         let challenge = transcript.squeeze();
//         // Partially evaluate the polynomial along the last variable using the challenge.
//         let challenge_bytes = challenge.into_bigint().to_bytes_be();
//         transcript.absorb(&challenge_bytes);
//         poly = poly.partial_evaluate((poly.number_of_variables - 1, challenge));
//     }
    
//     Proof {
//         claimed_sum,
//         round_polys,
//     }
// }

// fn verify<F: PrimeField>(proof: &Proof<F>, poly: &mut Polynomial<F>) -> bool {
//     if proof.round_polys.len() != poly.number_of_variables {
//         return false;
//     }

//     let mut challenges = vec![];

//     let mut transcript = Transcript::<Keccak256, F>::init(Keccak256::default());

//     transcript.absorb(

//         poly.evaluated_points
//             .iter()
//             .flat_map(|f| f.into_bigint().to_bytes_be())
//             .collect::<Vec<_>>()
//             .as_slice(),
//     );

//     transcript.absorb(proof.claimed_sum.into_bigint().to_bytes_be().as_slice());

//     let mut claimed_sum = proof.claimed_sum;

//     for round_poly in &proof.round_polys {
//         if claimed_sum != round_poly.iter().sum() {
//             return false;
//         }

//         transcript.absorb(
//             round_poly
//                 .iter()
//                 .flat_map(|f| f.into_bigint().to_bytes_be())
//                 .collect::<Vec<_>>()
//                 .as_slice(),
//         );

//         let challenge = transcript.squeeze();

//         challenges.push(challenge);
        
//         claimed_sum = round_poly[0] + challenge * (round_poly[1] - round_poly[0]);
//     }

//     if claimed_sum != poly.evaluate(challenges) {
//         return false;
//     }

//     true
// }

// #[cfg(test)]
// mod tests {
//     use crate::implementations::multilinear_polynomials::Polynomial;
//     use crate::implementations::fiat_shamir::{prove, verify};
//     use ark_bn254::Fr;


//     //check this test
//     // #[test]
//     // fn test_sumcheck() {
//     //     let mut poly = Polynomial::new(
//     //         vec![Fr::from(0), Fr::from(0), Fr::from(0), Fr::from(3), Fr::from(0), Fr::from(0), Fr::from(2), Fr::from(5)],
//     //         3
//     //     );
//     //     let proof = prove(poly.clone(), Fr::from(10));

//     //     dbg!(verify(&proof, &mut poly));
//     // }

//     #[test]
// fn test_sumcheck() {
//     let mut poly = Polynomial::new(
//         vec![
//             Fr::from(0), 
//             Fr::from(0), 
//             Fr::from(0), 
//             Fr::from(3), 
//             Fr::from(0), 
//             Fr::from(0), 
//             Fr::from(2), 
//             Fr::from(5)
//         ],
//         3,
//     );
    
//     let proof = prove(poly.clone(), Fr::from(10));
    
//     assert!(verify(&proof, &mut poly), "Proof verification failed");
// }

// }