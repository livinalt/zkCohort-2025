use protocol::implementations::shamir_secret_sharing::SecretSharing;
use protocol::implementations::transcript::Transcript;
use ark_bn254::Fq;

fn main() {

    //Calling the Shamir Secret Sharing implementation
    let secret_sharing = SecretSharing::new(Fq::from(4), 4, 10);
    let shares = secret_sharing.generate_shares();
    println!("=============================================================");
    println!("{:?}", shares);
    println!("=============================================================");
    
    // Calling the transcript implementation
    let mut transcript: Transcript<Fq> = Transcript::init();
    
    // Sample data to absorb into the transcript
    let data1 = b"Hello, ";
    let data2 = b"world!";
    
    // Absorb the first piece of data
    transcript.absorb(data1);
    
    // Absorb the second piece of data
    transcript.absorb(data2);
    
    // Squeeze the hash output from the transcript
    let result = transcript.squeeze();
    
    // Print the result (for demonstration purposes)
    println!("This is the Squeezed result: {:?}", result);
}

