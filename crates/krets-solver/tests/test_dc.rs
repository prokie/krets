#[cfg(test)]
mod tests {

    use std::{env, path::Path};

    use krets_parser::analyses::DcAnalysis;
    use krets_solver::solver::Solver;

    // Function to get the project root path at runtime
    fn get_manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap()
    }

    #[test]
    fn test_voltage_divider_dc() {
        let path = Path::new(&get_manifest_dir())
            .join("../../circuits/voltage_divider/voltage_divider.cir");
        let circuit = krets_parser::parse_circuit_description_file(&path).unwrap();
        let solver = Solver::new(circuit);

        let dc_analysis = DcAnalysis {
            element: "V1".to_string(),
            start: 0.0,
            stop: 1.0,
            step_size: 1.0,
        };

        let solution = solver.solve_dc(dc_analysis).unwrap();

        assert!((solution.get(0).unwrap().get("V(in)").unwrap() - 0.0).abs() < 1e-3);
        assert!((solution.get(0).unwrap().get("V(out)").unwrap() - 0.0).abs() < 1e-3);
        assert!((solution.get(0).unwrap().get("I(V1)").unwrap() - 0.0).abs() < 1e-3);

        assert!((solution.get(1).unwrap().get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((solution.get(1).unwrap().get("V(out)").unwrap() - 2.0 / 3.0).abs() < 1e-3);
        assert!((solution.get(1).unwrap().get("I(V1)").unwrap() - 1.0 / 3000.0).abs() < 1e-3);
    }
}
