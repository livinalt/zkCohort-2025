use ark_bn254::Fr;
use ark_ff::PrimeField;
use std::{marker::PhantomData};

use super::multilinear_polynomial::MultilinearPoly;



// GATES STRUCT AND IMPLEMENTATION
#[derive(Debug, Clone, Copy)]
pub struct Gates<F:PrimeField>{
    pub input_left:F,
    pub input_right:F,
    pub output:F,
    pub operator: Operator,
}
impl<F: PrimeField> Gates<F> {
    pub fn new_gate(input_left: F, input_right: F, operator: Operator) -> Self {
        let output = operator.use_operation(input_left, input_right);
        Self {
            input_left,
            input_right,
            output,
            operator,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Add,
    Mul,
}

impl Operator { 
    
    pub fn use_operation <F: PrimeField>(self, a:F , b:F) -> F{
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }    
}

// LAYER STRUCT AND IMPLEMENTATION
#[derive(Debug, Clone)]
pub struct Layers<F: PrimeField> {
    pub gates: Vec<Gates<F>>,
}

impl<F: PrimeField> Layers<F> {
    pub fn new_layer(gates: Vec<Gates<F>>) -> Self {
        Self { gates }
    }

    pub fn get_output_for_layers(&self) -> Vec<F> {
        self.gates.iter().map(|gates| gates.output).collect()
    }

    pub fn get_operators_of_layers(&self, ops: Operator) -> Vec<&Gates<F>> {
        self.gates.iter().filter(|gates| gates.operator == ops).collect()
    }

    pub fn get_add_mul_i(&self, op: Operator) -> MultilinearPoly<F> {
        let n_bits = self.get_no_bits_of_gates();
        let layer_size = 1 << n_bits;
        let mut poly_eval = vec![F::zero(); layer_size];

        let gate_values = self.gate_to_bits();
        for (gate_value, gate) in gate_values.into_iter().zip(&self.gates) {
            if gate.operator == op {
                poly_eval[gate_value] = F::one();
            }
        }

        MultilinearPoly::new(poly_eval)
    }
    

    // if we have one gate, then the number of bits ==> (abc) == (000)
    // if we have two gates, then the number of bits ==> (abbcc) == (0,00,00)
    // if we have three gates, then the number of bits ==> (aabbbccc) == (00,000,000)
    fn get_no_bits_of_gates(&self) -> u32 {
        let number_of_gates = self.gates.len();
        if number_of_gates == 0 {
            return 0;
        }
        (number_of_gates as f64).log2().ceil() as u32
    }



    pub fn gate_to_bits(&self) -> Vec<usize> {
        let n_bits = self.get_no_bits_of_gates();
        self.gates
            .iter()
            .enumerate()
            .map(|(i, _)| i % (1 << n_bits))
            .collect()
    }

}


// CIRCUIT STRUCT AND IMPLEMENTATION
// #[derive(Debug, Clone)]
pub struct Circuit<F:PrimeField>{
    pub layers:Vec<Layers<F>>,
}

impl<F: PrimeField> Circuit<F> {
    pub fn new_circuit(layers: Vec<Layers<F>>) -> Self {
        Self { layers }
    }

    pub fn evaluate_circuit(&self, inputs: Vec<F>) -> Vec<Vec<F>> {
        let mut result: Vec<Vec<F>> = Vec::new();
        let mut current_inputs: Vec<F> = inputs;

        for layer in &self.layers {
            let mut layer_outputs: Vec<F> = Vec::new();
            for gate in &layer.gates {
                let output_val = gate.operator.use_operation(gate.input_left, gate.input_right);
                layer_outputs.push(output_val);
            }
            result.push(layer_outputs.clone());
            current_inputs = layer_outputs;
        }
        result
    }
}


#[test]
fn test_circuit() {
    let input1 = Fr::from(1);
    let input2 = Fr::from(2);
    let input3 = Fr::from(3);

    let gate1: Gates<Fr> = Gates::new_gate(input1, input2, Operator::Add);
    let gate2: Gates<Fr> = Gates::new_gate(gate1.output, input3, Operator::Mul);

    let layer1: Layers<Fr> = Layers::new_layer(vec![gate1]);
    let layer2: Layers<Fr> = Layers::new_layer(vec![gate2]);

    let circuit: Circuit<Fr> = Circuit::new_circuit(vec![layer1, layer2]);

    let inputs: Vec<Fr> = vec![input1, input2, input3];
    let result = circuit.evaluate_circuit(inputs);

    // Expected output: [(1 + 2)], [(3) * 3] = [3], [9]
    // assert_eq!(result[0][0], Fr::from(3)); 
    assert_eq!(result[1][0], Fr::from(9)); 
}
