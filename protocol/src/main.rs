use protocol::implementations::shamir_secret_sharing::SecretSharing;
use ark_bn254::Fq;

fn main() {
    let secret_sharing = SecretSharing::new(Fq::from(4), 4, 10);
    let shares = secret_sharing.generate_shares();
    println!("{:?}", shares);
}
