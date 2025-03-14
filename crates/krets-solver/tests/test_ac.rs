#[cfg(test)]
mod tests {

    use std::{env, path::Path};

    use krets_solver::solver::Solver;

    // Function to get the project root path at runtime
    fn manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap()
    }

    // Function to get the circuits directory path
    fn circuits_dir() -> String {
        Path::new(&manifest_dir())
            .join("../../circuits/")
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_low_pass_filter_ac() {
        let path = Path::new(&circuits_dir()).join("low_pass_filter/low_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve_ac(1000.0);

        assert!((solution.get("V(out)").unwrap().0 - 2.533030e-08).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().1 - (-1.59155e-07)).abs() < 1e-3);
    }

    #[test]
    fn test_high_pass_filter_ac() {
        let path = Path::new(&circuits_dir()).join("high_pass_filter/high_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve_ac(1000.0);

        assert!((solution.get("frequency").unwrap().0 - 1.000000e+03).abs() < 1e-3);
        assert!((solution.get("frequency").unwrap().1 - 0.000000e+00).abs() < 1e-3);
        assert!((solution.get("I(L1)").unwrap().0 - 7.169568e-03).abs() < 1e-3);
        assert!((solution.get("I(L1)").unwrap().1 - (-4.50477e-03)).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().0 - 7.169568e-01).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().1 - (-4.50477e-01)).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap().0 - (-7.16957e-03)).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap().1 - 4.504772e-03).abs() < 1e-3);
    }

    #[test]
    fn test_voltage_divider_ac() {
        let path =
            Path::new(&manifest_dir()).join("../../circuits/voltage_divider/voltage_divider.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);

        let solution = solver.solve_ac(1000.0);

        assert!((solution.get("V(in)").unwrap().0 - 0.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().0 - 0.0).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap().0 - 0.0).abs() < 1e-3);

        assert!((solution.get("V(in)").unwrap().1 - 0.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().1 - 0.0).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap().1 - 0.0).abs() < 1e-3);
    }
}
