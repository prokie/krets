#[cfg(test)]
mod tests {
    use krets_parser::{elements::Element, parse_circuit_description, prelude::*};

    #[test]
    fn test_parse_empty_netlist() {
        let netlist = "";
        let result = parse_circuit_description(netlist);
        assert!(matches!(result, Err(Error::EmptyNetlist)));
    }

    #[test]
    fn test_parse_voltage_source() {
        let netlist = "V1 1 0 5";
        let result = parse_circuit_description(netlist);
        assert!(result.is_ok());

        let netlist = result.unwrap();
        assert_eq!(netlist.elements.len(), 1);

        match &netlist.elements[0] {
            Element::VoltageSource(vs) => {
                assert_eq!(vs.name, 1);
                assert_eq!(vs.plus, "1");
                assert_eq!(vs.minus, "0");
                assert_eq!(vs.value, 5.0);
            }
            _ => panic!("Expected a voltage source element"),
        }
    }

    #[test]
    fn test_parse_with_comment_line() {
        let netlist = "% This is a comment\nV1 1 0 5";
        let circuit = parse_circuit_description(netlist);
        assert!(circuit.is_ok());

        let netlist = circuit.unwrap();
        assert_eq!(netlist.elements.len(), 1);

        match &netlist.elements[0] {
            Element::VoltageSource(vs) => {
                assert_eq!(vs.name, 1);
                assert_eq!(vs.plus, "1");
                assert_eq!(vs.minus, "0");
                assert_eq!(vs.value, 5.0);
            }
            _ => panic!("Expected a voltage source element"),
        }
    }

    #[test]
    fn test_parse_case_insensitive() {
        let netlist = "v1 1 0 5";
        let circuit = parse_circuit_description(netlist);
        assert!(circuit.is_ok());

        let netlist = circuit.unwrap();
        assert_eq!(netlist.elements.len(), 1);

        match &netlist.elements[0] {
            Element::VoltageSource(vs) => {
                assert_eq!(vs.name, 1);
                assert_eq!(vs.plus, "1");
                assert_eq!(vs.minus, "0");
                assert_eq!(vs.value, 5.0);
            }
            _ => panic!("Expected a voltage source element"),
        }
    }

    #[test]
    fn test_parse_invalid_format() {
        let netlist = "V1 1 0";
        let circuit = parse_circuit_description(netlist);
        assert!(matches!(circuit, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_current_source() {
        let netlist = "i1 1 0 5";
        let circuit = parse_circuit_description(netlist);
        assert!(circuit.is_ok());

        let netlist = circuit.unwrap();
        assert_eq!(netlist.elements.len(), 1);

        match &netlist.elements[0] {
            Element::CurrentSource(cs) => {
                assert_eq!(cs.name, 1);
                assert_eq!(cs.plus, "1");
                assert_eq!(cs.minus, "0");
                assert_eq!(cs.value, 5.0);
            }
            _ => panic!("Expected a current source element"),
        }
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
R3 5 2 50 G2 % this is a group 2 element
R4 5 6 0.1
R5 2 6 1.5
R6 3 4 0.1
R7 8 0 1e3
R8 4 0 10 G2 % this is a group 2 element";
        let circuit = parse_circuit_description(netlist);
        assert!(circuit.is_ok());

        let circuit = circuit.unwrap();
        assert_eq!(circuit.elements.len(), 13);
        assert_eq!(circuit.nodes.len(), 9);
    }
}
