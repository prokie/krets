#[cfg(test)]
mod tests {

    use std::{env, path::Path};

    use krets_solver::Solver;

    // Function to get the project root path at runtime
    fn get_manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap()
    }

    //     #[test]
    //     fn test_assemble_mna_system() {
    //         // This is taken from Figure 2.35 in the book.
    //         let circuit_description = "
    // V1 5 0 2
    // V2 3 2 0.2
    // V3 7 6 2
    // I1 4 8 1e-3
    // I2 0 6 1e-3
    // R1 1 5 1.5
    // R2 1 2 1
    // R3 5 2 50 G2 % this is a group 2 element
    // R4 5 6 0.1
    // R5 2 6 1.5
    // R6 3 4 0.1
    // R7 8 0 1e3
    // R8 4 0 10 G2 % this is a group 2 element
    // ";
    //         let circuit = parser::parse_circuit_description(circuit_description).unwrap();
    //         let solver = Solver::new(circuit);
    //         solver.assemble_mna_system();
    //     }

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
        let circuit = krets_parser::parse_circuit_description(circuit_description).unwrap();
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
        let path = Path::new(&get_manifest_dir())
            .join("../../circuits/voltage_divider/voltage_divider.cir");
        let circuit = krets_parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);
        let solution = solver.solve();

        assert!((solution.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap() - 0.6667).abs() < 1e-3);
        assert!((solution.get("I(V1)").unwrap() - 1. / 3000.).abs() < 1e-3);
    }
}
