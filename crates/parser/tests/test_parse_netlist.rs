#[cfg(test)]
mod tests {
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
        assert!(result.elements.len() == 2);
        assert!(matches!(result.elements[0], ElementKind::Resistor(_)));
        assert!(matches!(result.elements[0].name, "R1"));
    }
}
