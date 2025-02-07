use ark_ff::PrimeField;
use std::marker::PhantomData;

pub enum Operator {
    Add,
    Mul,
}

pub struct Gate {
    pub left_val: usize,
    pub right_val: usize,
    pub operator: Operator,
}

pub struct Layer {
    pub gates: Vec<Gate>,
}

pub struct Circuit<F: PrimeField> {
    pub layers: Vec<Layer>,
    _phantom: PhantomData<F>,
}

impl Gate {
    pub fn new(left_val: usize, right_val: usize, operator: Operator) -> Self {
        Self {
            left_val,
            right_val,
            operator,
        }
    }
}

impl Layer {
    pub fn new(gates: Vec<Gate>) -> Self {
        Self { gates }
    }
}

impl<F: PrimeField> Circuit<F> {
    pub fn new(layers: Vec<Layer>) -> Self {
        Self {
            layers,
            _phantom: PhantomData,
        }
    }

    pub fn evaluate(&self, inputs: &[F]) -> F {
        let mut current_values = inputs.to_vec();

        for layer in &self.layers {
            let mut next_values = Vec::new();

            for gate in &layer.gates {

                let left_val = current_values[gate.left_val];
                let right_val = current_values[gate.right_val];

                let result = match gate.operator {
                    Operator::Add => left_val + right_val,
                    Operator::Mul => left_val * right_val,
                };

                next_values.push(result);
                
            }
            current_values = next_values;
        }

        // The final result is the first element of the last layer's output
        current_values[0]
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ark_bn254::Fr; 
    use ark_ff::UniformRand;
    use ark_std::test_rng; 

    #[test]
    fn test_circuit_evaluation() {

        // Layer 1: Two gates
        let layer1_gates = vec![
            Gate::new(0, 1, Operator::Add), 
            Gate::new(2, 3, Operator::Mul), 
        ];
        let layer1 = Layer::new(layer1_gates);

        // Layer 2: One gate
        let layer2_gates = vec![
            Gate::new(0, 1, Operator::Mul), 
        ];
        let layer2 = Layer::new(layer2_gates);

        let circuit = Circuit::new(vec![layer1, layer2]);

        let inputs = vec![
            Fr::from(2),  
            Fr::from(3),  
            Fr::from(4),  
            Fr::from(5),  
        ];

        let expected_result = Fr::from((2 + 3) * (4 * 5));

        let actual_result = circuit.evaluate(&inputs);
        println!("This is the actual result: {} and the expected result: {}", actual_result, expected_result);
        assert_eq!(actual_result, expected_result);

    }

    #[test]
    fn test_circuit_with_random_inputs() {
        let mut rng = test_rng(); 

        // Generate random inputs
        let input: Vec<Fr> = (0..4).map(|_| Fr::rand(&mut rng)).collect();  

        let gate1 = Gate::new(0, 1, Operator::Add);
        let gate2 = Gate::new(2, 3, Operator::Mul);

        let layer1 = Layer::new(vec![gate1, gate2]);

        let gate3 = Gate::new(0, 1, Operator::Mul); 
        let layer2 = Layer::new(vec![gate3]);

        let circuit = Circuit::new(vec![layer1, layer2]);

        // Calculate expected result 
        let expected_result = (input[0] + input[1]) * (input[2] * input[3]);

        let actual_result = circuit.evaluate(&input);
        
        println!("This is the actual result: {} and the expected result: {}", actual_result, expected_result);

        assert_eq!(actual_result, expected_result);
    }

}


fn main() {
    println!("Hello, Circuits world!");
}