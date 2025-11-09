#[cfg(test)]
mod tests {
    use krets_parser::analyses::{Analysis, DcAnalysis};
    use krets_solver::{config::SolverConfig, solver::Solver};
    use std::{env, path::Path};

    // Function to get the project root path at runtime
    fn manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string())
    }

    // Function to get the circuits directory path
    fn circuits_dir() -> String {
        // Adjust the path to navigate from the crate's root to the workspace root's circuits dir
        Path::new(&manifest_dir())
            .parent() // Go up from crates/krets-solver
            .and_then(Path::parent) // Go up from crates
            .unwrap()
            .join("circuits/")
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_voltage_divider_dc() {
        let path = Path::new(&circuits_dir()).join("voltage_divider/voltage_divider.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let mut solver = Solver::new(circuit, config);

        let dc_analysis = DcAnalysis {
            element: "V1".to_string(),
            start: 0.0,
            stop: 1.0,
            step_size: 1.0,
        };

        let solution = solver.solve(Analysis::Dc(dc_analysis)).unwrap().into_dc();

        assert_eq!(solution.len(), 2); // Should have results for 0V and 1V

        // Check results for 0V sweep point
        let first_result = &solution[0];
        assert!((first_result.get("V(in)").unwrap() - 0.0).abs() < 1e-3);
        assert!((first_result.get("V(out)").unwrap() - 0.0).abs() < 1e-3);
        assert!((first_result.get("I(V1)").unwrap() - 0.0).abs() < 1e-3);

        // Check results for 1V sweep point
        let second_result = &solution[1];
        assert!((second_result.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
        assert!((second_result.get("V(out)").unwrap() - 2.0 / 3.0).abs() < 1e-3);
        assert!((second_result.get("I(V1)").unwrap() - (-1.0 / 3000.0)).abs() < 1e-4);
    }

    // #[test]
    // fn test_inverter() {
    //     let path = Path::new(&circuits_dir()).join("inverter/dc/inverter.cir");
    //     let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
    //     let config = SolverConfig::default();
    //     let mut solver = Solver::new(circuit, config);

    //     let dc_analysis = DcAnalysis {
    //         element: "V1".to_string(),
    //         start: 0.0,
    //         stop: 1.0,
    //         step_size: 1.0,
    //     };

    //     let solution = solver.solve(Analysis::Dc(dc_analysis)).unwrap().into_dc();

    //     assert_eq!(solution.len(), 2); // Should have results for 0V and 1V

    //     // Check results for 0V sweep point
    //     let first_result = &solution[0];
    //     assert!((first_result.get("V(in)").unwrap() - 0.0).abs() < 1e-3);
    //     assert!((first_result.get("V(out)").unwrap() - 0.0).abs() < 1e-3);
    //     assert!((first_result.get("I(V1)").unwrap() - 0.0).abs() < 1e-3);

    //     // Check results for 1V sweep point
    //     let second_result = &solution[1];
    //     assert!((second_result.get("V(in)").unwrap() - 1.0).abs() < 1e-3);
    //     assert!((second_result.get("V(out)").unwrap() - 2.0 / 3.0).abs() < 1e-3);
    //     assert!((second_result.get("I(V1)").unwrap() - (-1.0 / 3000.0)).abs() < 1e-4);
    // }
}
