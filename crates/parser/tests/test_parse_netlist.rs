#[cfg(test)]
mod tests {
    use parser::{elements::Element, parse_circuit_description, prelude::*};

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
                assert_eq!(vs.node_plus, "1");
                assert_eq!(vs.node_minus, "0");
                assert_eq!(vs.value, 5.0);
            }
        }
    }

    #[test]
    fn test_parse_with_comment_line() {
        let netlist = "% This is a comment\nV1 1 0 5";
        let result = parse_circuit_description(netlist);
        assert!(result.is_ok());

        let netlist = result.unwrap();
        assert_eq!(netlist.elements.len(), 1);

        match &netlist.elements[0] {
            Element::VoltageSource(vs) => {
                assert_eq!(vs.name, 1);
                assert_eq!(vs.node_plus, "1");
                assert_eq!(vs.node_minus, "0");
                assert_eq!(vs.value, 5.0);
            }
        }
    }
}
