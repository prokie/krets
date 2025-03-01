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
    fn test_parse_case_insensitive() {
        let netlist = "v1 1 0 5";
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
    fn test_parse_invalid_format() {
        let netlist = "V1 1 0";
        let result = parse_circuit_description(netlist);
        assert!(matches!(result, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_current_source() {
        let netlist = "i1 1 0 5";
        let result = parse_circuit_description(netlist);
        dbg!(&result);
        assert!(result.is_ok());

        let netlist = result.unwrap();
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
}
