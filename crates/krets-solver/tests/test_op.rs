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
    fn test_circuit_simulation_farid_n_najm() {
        let path = Path::new(&circuits_dir())
            .join("circuit_simulation_farid_n_najm/circuit_simulation_farid_n_najm.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();

        let solver = Solver::new(circuit);
        let solution = solver.solve();
        assert!((solution.get("V(4)").unwrap() - 1.9888).abs() < 1e-3);
        assert!((solution.get("V(8)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get("V(3)").unwrap() - 2.00879).abs() < 1e-3);
        assert!((solution.get("V(2)").unwrap() - 1.80879).abs() < 1e-3);
        assert!((solution.get("V(6)").unwrap() - 1.98814).abs() < 1e-3);
        assert!((solution.get("V(5)").unwrap() - 2.0).abs() < 1e-3);
        assert!((solution.get("V(1)").unwrap() - 1.88527).abs() < 1e-3);
        assert!((solution.get("V(7)").unwrap() - 3.98814).abs() < 1e-3);
        assert!((solution.get("I(R8)").unwrap() - 198.88e-3).abs() < 1e-3);
        assert!((solution.get("I(R3)").unwrap() - 3.82e-3).abs() < 1e-3);
        assert!((solution.get("I(V3)").unwrap() - 0.0).abs() < 1e-3);
        assert!((solution.get("I(V2)").unwrap() - (-199.88e-3)).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap() - (-198.88e-3)).abs() < 1e-3);
    }

    #[test]
    fn test_case_1() {
        // This is taken from website.
        let circuit_description = "
V1 2 1 32
R1 1 0 2
R2 2 3 4
R3 2 0 8
V2 3 0 20
    ";
        let circuit = krets_parser::parser::parse_circuit_description(circuit_description).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(1)").unwrap() - (-8.0)).abs() < 1e-3);
        assert!((solution.get("V(2)").unwrap() - 24.0).abs() < 1e-3);
        assert!((solution.get("V(3)").unwrap() - 20.0).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap() - (-4.0)).abs() < 1e-3);
        assert!((solution.get("I(V2)").unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_voltage_divider() {
        let path = Path::new(&circuits_dir()).join("voltage_divider/voltage_divider.cir");
        dbg!(&path);
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap() - 2.0 / 3.0).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap() - 1. / 3000.).abs() < 1e-3);
    }

    #[test]
    fn test_low_pass_filter_op() {
        let path = Path::new(&circuits_dir()).join("low_pass_filter/low_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_high_pass_filter_op() {
        let path = Path::new(&circuits_dir()).join("high_pass_filter/high_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap() - 1.0).abs() < 1e-3);
    }

    #[test]
    fn test_basic_001_op() {
        let path = Path::new(&circuits_dir()).join("basic_001/basic_001.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(1)").unwrap() - 3.0).abs() < 1e-3);
        assert!((solution.get("V(2)").unwrap() - 0.5).abs() < 1e-3);
        assert!((solution.get("I(V4)").unwrap() - (-0.5)).abs() < 1e-3);
    }
}
