#[cfg(test)]
mod tests {
    use krets_parser::{parser::parse_circuit_description, prelude::*};

    #[test]
    fn test_parse_empty_netlist() {
        let netlist = "";
        let result = parse_circuit_description(netlist);
        assert!(matches!(result, Err(Error::EmptyNetlist)));
    }

    #[test]
    fn test_parse_netlist() {
        let netlist = "V1 5 0 2
V2 3 2 0.2
V3 7 6 2
I1 4 8 1e-3
I2 0 6 1e-3
R1 1 5 1.5
R2 1 2 1
R3 5 2 50
R4 5 6 0.1
R5 2 6 1.5
R6 3 4 0.1
R7 8 0 1e3
R8 4 0 10";
        let circuit = parse_circuit_description(netlist);

        if let Err(e) = &circuit {
            println!("Parsing failed with error: {:?}", e);
        }
        assert!(circuit.is_ok());

        let circuit = circuit.unwrap();
        assert_eq!(circuit.elements.len(), 13);
        assert_eq!(circuit.nodes.len(), 9);
    }
}
