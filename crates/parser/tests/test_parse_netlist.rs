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
}
