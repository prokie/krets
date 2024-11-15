#[cfg(test)]
mod tests {
    use parser::element::Element;
    use parser::parse_netlist;
    use parser::prelude::*;

    #[test]
    fn test_parse_empty_netlist() {
        let netlist = "";
        let result = parse_netlist(netlist);
        assert!(matches!(result, Err(Error::EmptyNetlist)));
    }

    #[test]
    fn test_parse_invalid_format() {
        let netlist = "Title\nInvalid\n";
        let result = parse_netlist(netlist);
        assert!(matches!(result, Err(Error::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_resistor() {
        let netlist = "Title\nR1 1 2 100\n";
        let result = parse_netlist(netlist).unwrap();
        assert!(result.elements.len() == 1);
        assert!(matches!(result.elements[0], Element::Resistor(_)));
        let Element::Resistor(element) = &result.elements[0] else {
            panic!("Expected resistor element")
        };
        assert!(element.name == "1");
    }
    #[test]
    fn test_parse_voltage_source() {
        let netlist = "Title\nV1 1 0 10\n";
        let result = parse_netlist(netlist).unwrap();
        assert!(result.elements.len() == 1);
        assert!(matches!(result.elements[0], Element::VoltageSource(_)));
        let Element::VoltageSource(element) = &result.elements[0] else {
            panic!("Expected voltage source element")
        };
        assert!(element.name == "1");
    }

    #[test]
    fn test_parse_voltage_source_long_name() {
        let netlist = "Title\nVthis_is_a_long_name 1 0 10\n";
        let result = parse_netlist(netlist).unwrap();
        assert!(result.elements.len() == 1);
        assert!(matches!(result.elements[0], Element::VoltageSource(_)));
        let Element::VoltageSource(element) = &result.elements[0] else {
            panic!("Expected voltage source element")
        };
        assert!(element.name == "this_is_a_long_name");
    }

    #[test]
    fn test_parse_current_source() {
        let netlist = "Title\nI1 1 0 10\n";
        let result = parse_netlist(netlist).unwrap();
        assert!(result.elements.len() == 1);
        assert!(matches!(result.elements[0], Element::CurrentSource(_)));
        let Element::CurrentSource(element) = &result.elements[0] else {
            panic!("Expected current source element")
        };
        assert!(element.name == "1");
    }

    #[test]
    fn test_parse_capacitor() {
        let netlist = "Title\nC1 1 2 100\n";
        let result = parse_netlist(netlist).unwrap();
        assert!(result.elements.len() == 1);
        assert!(matches!(result.elements[0], Element::Capacitor(_)));
        let Element::Capacitor(element) = &result.elements[0] else {
            panic!("Expected capacitor element")
        };
        assert!(element.name == "1");
    }

    #[test]
    fn test_parse_complex_netlist() {
        let netlist = "
    Title
    R1 1 2 100
    R2 2 3 200
    C1 1 3 10u
    L1 3 0 1m
    V1 1 0 10
    I1 2 0 5
    ";

        let result = parse_netlist(netlist).unwrap();

        assert!(matches!(result.elements[0], Element::Resistor(_)));
        assert!(result.elements[0].name() == "1");

        assert!(matches!(result.elements[1], Element::Resistor(_)));
        assert!(result.elements[1].name() == "2");

        assert!(matches!(result.elements[2], Element::Capacitor(_)));
        assert!(result.elements[2].name() == "1");

        assert!(matches!(result.elements[3], Element::Inductor(_)));
        assert!(result.elements[3].name() == "1");

        assert!(matches!(result.elements[4], Element::VoltageSource(_)));
        assert!(result.elements[4].name() == "1");

        assert!(matches!(result.elements[5], Element::CurrentSource(_)));
        assert!(result.elements[5].name() == "1");
    }
}
