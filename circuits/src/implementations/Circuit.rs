use ark_bn254::Fr;
use ark_ff::PrimeField;
use std::{marker::PhantomData};



struct Gates<F:PrimeField>{
    input_left:F,
    input_right:F,
    output:F,
    operator: Operator,
}
impl<F: PrimeField> Gates<F> {
    fn new_gate(input_left: F, input_right: F, operator: Operator) -> Self {
        let output = operator.use_operation(input_left, input_right);
        Self {
            input_left,
            input_right,
            output,
            operator,
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Operator {
    Add,
    Mul,
}

impl Operator { 
    
    fn use_operation <F: PrimeField>(self, a:F , b:F) -> F{
        match self {
            Operator::Add => a + b,
            Operator::Mul => a * b,
        }
    }    
}

// #[derive(Debug, Clone)]
struct Layers<F: PrimeField> {
    gates: Vec<Gates<F>>,
}

impl<F: PrimeField> Layers<F> {
    fn new_layer(gates: Vec<Gates<F>>) -> Self {
        Self { gates }
    }

    fn get_output_for_layers(&self) -> Vec<F> {
        self.gates.iter().map(|gates| gates.output).collect()
    }

    fn get_operators_of_layers(&self, ops: Operator) -> Vec<&Gates<F>> {
        self.gates.iter().filter(|gates| gates.operator == ops).collect()
    }
    

    // if we have one gate, then the number of bits ==> (abc) == (000)
    // if we have two gates, then the number of bits ==> (abbcc) == (0,00,00)
    // if we have three gates, then the number of bits ==> (aabbbccc) == (00,000,000)
    fn get_no_bits_of_gates(&self) -> u32{
        let number_of_gates = self.gates.len();


        if number_of_gates == 1{
            return 3;
        }

        else{
            let number_of_bits_of_gates = number_of_gates.ilog2();

            let number_of_gates_log = number_of_bits_of_gates.ilog2();
            let number_of_bits = number_of_gates_log + 1;
            return number_of_gates_log + (number_of_bits * 2);
          };
    }


    fn gate_to_bits(&self) -> Vec<usize> {
        self.gates
            .iter()
            .enumerate()
            .map(|(i, _)| 5 * i + 1) 
            .collect()
    }

}



struct Circuit<F:PrimeField>{
    layers:Vec<Layers<F>>,
}

impl<F: PrimeField> Circuit<F> {
    fn new_circuit(layers: Vec<Layers<F>>) -> Self {
        Self { layers }
    }

    fn evaluate_circuit(&self, inputs:Vec<F>) -> Vec<Vec<F>> {
        
        let mut result: Vec<Vec<F>> = Vec::new();
        let mut current_inputs: Vec<F> = inputs;

        for layer in &self.layers {
            let mut layer_outputs: Vec<F> = Vec::new();
            for gate in &layer.gates {
                let output_val = gate.operator.use_operation(gate.input_left, gate.input_left);
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
    assert_eq!(result[0][0], Fr::from(3)); 
    assert_eq!(result[1][0], Fr::from(9)); 
}
