use ark_bn254::Fr;
use ark_ff::PrimeField;
use std::{marker::PhantomData};

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Operator {
    Add,
    Mul,
}

#[derive(Debug, Clone)]
pub struct Gate<F: PrimeField> {
    pub left_val: F,
    pub right_val: F,
    pub output_val: F,
    pub operator: Operator,
}

#[derive(Debug)]
pub struct Layer<F: PrimeField> {
    pub gates: Vec<Gate<F>>,
}

#[derive(Debug)]
pub struct Circuit<F: PrimeField> {
    pub layers: Vec<Layer<F>>,
}

impl Operator {
    pub fn use_operator<F: PrimeField>(self, a: F, b: F) -> F {
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }
}

impl<F: PrimeField> Gate<F> {
    pub fn new(left_val: F, right_val: F, operator: Operator) -> Self {
        let output_val = operator.use_operator(left_val, right_val);
        Self {
            left_val,
            right_val,
            output_val,
            operator,
        }
    }
}

impl<F: PrimeField> Layer<F> {
    pub fn new(gates: Vec<Gate<F>>) -> Self {
        Self { gates }
    }

    fn get_output_values(&self) -> Vec<F> {
        self.gates.iter().map(|gate| gate.output_val).collect()
    }


    fn get_indicator_poly(&self, op: Operator) -> Vec<u8> {
        let n_bits = self.get_bits_for_gates();
        let layer_size = 1 << n_bits;
        let mut poly_eval = vec![0; layer_size];

        let gate_indices = self.gate_to_bits();
        for (gate_index, gate) in gate_indices.into_iter().zip(&self.gates) {
            if gate.operator == op {
                poly_eval[gate_index] = 1;
            }
        }
        poly_eval
    }


    fn get_bits_for_gates(&self) -> u32 {
        let n_gates = self.gates.len();
        assert!(n_gates > 0, "There must be at least one gate in the layer.");

        if n_gates == 1 {
            3 
        } else {
            let n_gates_log = n_gates.ilog2();
            let n_bits = n_gates_log + 1;
            n_gates_log + (n_bits * 2)
        }
    }


    fn gate_to_bits(&self) -> Vec<usize> {
        self.gates
            .iter()
            .enumerate()
            .map(|(i, _)| 5 * i + 1) 
            .collect()
    }
}


impl<F: PrimeField> Circuit<F> {
    pub fn new(layers: Vec<Layer<F>>) -> Self {
        Self {
            layers
        }
    }

    pub fn evaluate(&mut self, inputs: Vec<F>) -> Vec<Vec<F>> {
        let mut result: Vec<Vec<F>> = Vec::new();
        let mut current_inputs: Vec<F> = inputs;

        for layer in &self.layers {
            let mut layer_outputs: Vec<F> = Vec::new();
            for gate in &layer.gates {
                let output_val = gate.operator.use_operator(gate.left_val, gate.right_val);
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

    let gate1: Gate<Fr> = Gate::new(input1, input2, Operator::Add);
    
    let gate2: Gate<Fr> = Gate::new(gate1.output_val, input3, Operator::Mul);

    let layer1: Layer<Fr> = Layer::new(vec![gate1]);
    let layer2: Layer<Fr> = Layer::new(vec![gate2]);

    let mut circuit: Circuit<Fr> = Circuit::new(vec![layer1, layer2]);

    let inputs: Vec<Fr> = vec![input1, input2, input3];
    let result = circuit.evaluate(inputs);

    // Expected output: [(1 + 2)], [(3) * 3] = [3], [9]
    assert_eq!(result[0][0], Fr::from(3)); 
    assert_eq!(result[1][0], Fr::from(9)); 
}

