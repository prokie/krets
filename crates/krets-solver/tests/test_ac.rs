#[cfg(test)]
mod tests {
    use krets_parser::analyses::Analysis;
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
    fn test_low_pass_filter_ac() {
        let path = Path::new(&circuits_dir()).join("low_pass_filter/low_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let ac_analysis = krets_parser::analyses::AcAnalysis {
            fstart: 1000.0,
            sweep: krets_parser::analyses::AcSweep::Linear { total_points: 1 },
            fstop: 1000.0,
        };
        let mut solver = Solver::new(circuit, config);
        let analysis = Analysis::Ac(ac_analysis);
        let solution = solver
            .solve(analysis)
            .unwrap()
            .into_ac()
            .first()
            .unwrap()
            .clone();
        assert!((solution.get("V(out)").unwrap().re - 2.470452e-02).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().im - (-1.55223e-01)).abs() < 1e-3);
    }

    #[test]
    fn test_high_pass_filter_ac() {
        let path = Path::new(&circuits_dir()).join("high_pass_filter/high_pass_filter.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let ac_analysis = krets_parser::analyses::AcAnalysis {
            fstart: 1000.0,
            sweep: krets_parser::analyses::AcSweep::Linear { total_points: 1 },
            fstop: 1000.0,
        };
        let mut solver = Solver::new(circuit, config);
        let analysis = Analysis::Ac(ac_analysis);
        let solution = solver
            .solve(analysis)
            .unwrap()
            .into_ac()
            .first()
            .unwrap()
            .clone();

        assert!((solution.get("frequency").unwrap().re - 1000.0).abs() < 1e-3);
        assert!((solution.get("frequency").unwrap().im - 0.0).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().re - 7.169568e-01).abs() < 1e-3);
        assert!((solution.get("V(out)").unwrap().im - (-4.50477e-01)).abs() < 1e-3); // Corrected expected sign
        assert!((solution.get("I(V1)").unwrap().re - (-7.16957e-03)).abs() < 1e-4);
        assert!((solution.get("I(V1)").unwrap().im - 4.504772e-03).abs() < 1e-4);
    }
}
