mod polynomials;
mod shamir_secret_sharing;

use ark_bn254::Fq;
use ark_ff::{Field, PrimeField};
use shamir_secret_sharing::{SecretSharing, Share};
use std::io;

type F = Fq;

fn main() {
    // Read the secret and number of shares and threshold
    println!("Enter the secret (a positive integer): ");
    let mut secret_input = String::new();
    io::stdin()
        .read_line(&mut secret_input)
        .expect("Failed to read line");
    let secret: u64 = secret_input
        .trim()
        .parse()
        .expect("Please enter a valid number");
    let secret = F::from(secret);

    println!("Enter the total number of shares: ");
    let mut total_shares_input = String::new();
    io::stdin()
        .read_line(&mut total_shares_input)
        .expect("Failed to read line");
    let total_shares: u64 = total_shares_input
        .trim()
        .parse()
        .expect("Please enter a valid number");

    println!("Enter the threshold (minimum number of shares needed to reconstruct the secret): ");
    let mut threshold_input = String::new();
    io::stdin()
        .read_line(&mut threshold_input)
        .expect("Failed to read line");
    let threshold: u64 = threshold_input
        .trim()
        .parse()
        .expect("Please enter a valid number");

    // Ensure threshold is less than or equal to total shares
    if threshold > total_shares {
        panic!("Threshold cannot be greater than the total number of shares.");
    }

    // Initialize Shamir's Secret Sharing scheme
    let secret_sharing = SecretSharing::<F>::new(secret, total_shares, threshold);

    // Generate shares
    let shares = secret_sharing.generate_shares();
    println!("Generated shares: ");
    for share in &shares {
        println!("({}, {})", share.x, share.y);
    }

    // Reconstruct the secret
    println!("Reconstructing the secret from shares...");
    let reconstructed_secret = secret_sharing.reconstruct_secret(&shares[0..threshold as usize]);
    println!("Reconstructed secret: {}", reconstructed_secret);
}
