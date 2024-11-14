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
        assert!(result.elements[0].name() == "R1");
        let Element::Resistor(element) = &result.elements[0] else {
            panic!("Expected resistor element")
        };

        println!("Name: {}", element.name);
    }
}
