#[cfg(test)]
mod tests {
    use krets_parser::analyses::{Analysis, TransientAnalysis};
    use krets_solver::{AnalysisResult, config::SolverConfig, solver::Solver};
    use std::{env, path::Path};
    // Function to get the project root path at runtime
    fn manifest_dir() -> String {
        env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string())
    }

    #[allow(dead_code)]
    fn print_results_to_console(result: &AnalysisResult) {
        match result {
            AnalysisResult::Op(op_solution) => {
                let mut sorted_results: Vec<_> = op_solution.iter().collect();
                sorted_results.sort_by_key(|(k, _)| *k);

                println!("{:<15} | {:<15}", "Node/Branch", "Value");
                println!("{:-<15}-+-{:-<15}", "", "");

                for (node_or_branch, value) in sorted_results {
                    let unit = if node_or_branch.starts_with('V') {
                        "V"
                    } else {
                        "A"
                    };
                    println!("{:<15} | {:>14.6e} {}", node_or_branch, value, unit);
                }
            }
            AnalysisResult::Dc(dc_solution) => {
                if dc_solution.is_empty() {
                    println!("DC sweep produced no results.");
                    return;
                }
                // Get headers from the first result, sorted for consistent order
                let mut headers: Vec<_> = dc_solution[0].keys().collect();
                headers.sort();

                // Print header
                for header in &headers {
                    print!("{:<18}", header);
                }
                println!();
                println!("{:-<width$}", "", width = headers.len() * 18);

                // Print rows
                for step_result in dc_solution {
                    for header in &headers {
                        if let Some(value) = step_result.get(*header) {
                            print!("{:<18.6e}", value);
                        } else {
                            print!("{:<18}", "N/A");
                        }
                    }
                    println!();
                }
            }
            AnalysisResult::Ac(ac_sweep_solution) => {
                // ac_sweep_solution is now Vec<HashMap<String, c64>>
                if ac_sweep_solution.is_empty() {
                    println!("AC sweep produced no results.");
                    return;
                }

                // Print header once
                println!(
                    "{:<18} | {:<15} | {:<20} | {:<20}",
                    "Frequency (Hz)", "Node/Branch", "Magnitude", "Phase (deg)"
                );
                println!("{:-<18}-+-{:-<15}-+-{:-<20}-+-{:-<20}", "", "", "", "");

                // Iterate through each frequency step's results
                for ac_solution_step in ac_sweep_solution {
                    // Extract frequency (assuming it's stored as "frequency" key)
                    let frequency = ac_solution_step.get("frequency").map_or(f64::NAN, |c| c.re); // Get real part for frequency

                    // Sort results for the current frequency step
                    let mut sorted_results: Vec<_> = ac_solution_step
                        .iter()
                        .filter(|(k, _)| **k != "frequency") // Exclude the frequency entry itself
                        .collect();
                    sorted_results.sort_by_key(|(k, _)| *k);

                    // Print results for the current frequency
                    for (node, value) in sorted_results {
                        let (mag, phase_deg) = (value.norm(), value.arg().to_degrees());
                        println!(
                            "{:<18.6e} | {:<15} | {:>19.6e} | {:>19.6e}",
                            frequency, node, mag, phase_deg
                        );
                    }
                    println!("{:-<18}-+-{:-<15}-+-{:-<20}-+-{:-<20}", "", "", "", ""); // Separator between frequencies
                }
            }
            AnalysisResult::Transient(tran_solution) => {
                if tran_solution.is_empty() {
                    println!("Transient analysis produced no results.");
                    return;
                }
                // Get headers from the first result, sorted for consistent order
                let mut headers: Vec<_> = tran_solution[0].keys().collect();
                headers.sort();

                // Print header
                for header in &headers {
                    print!("{:<18}", header);
                }
                println!();
                println!("{:-<width$}", "", width = headers.len() * 18);

                // Print rows
                for step_result in tran_solution {
                    for header in &headers {
                        if let Some(value) = step_result.get(*header) {
                            print!("{:<18.6e}", value);
                        } else {
                            print!("{:<18}", "N/A");
                        }
                    }
                    println!();
                }
            }
        }
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
    fn test_dual_rc_ladder_transient() {
        let path = Path::new(&circuits_dir()).join("dual_rc_ladder/dual_rc_ladder.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let mut solver = Solver::new(circuit, config);

        let tran_analysis = TransientAnalysis {
            time_step: 50e-6, // 50us
            stop_time: 50e-3, // 50ms
        };

        let solution = solver.solve(Analysis::Transient(tran_analysis)).unwrap();
        // print_results_to_console(&solution);
        let transient_solution = solution.clone().into_transient();

        let result_t0 = &transient_solution[0];
        assert!((result_t0.get("V(in)").unwrap() - 0.0).abs() < 1e-3);
        assert!((result_t0.get("V(out)").unwrap() - 0.0).abs() < 1e-3);

        let result_last = transient_solution.last().unwrap();
        assert!((result_last.get("V(out)").unwrap() - 0.989).abs() < 1e-3);
    }

    #[test]
    fn test_rectifier() {
        let path = Path::new(&circuits_dir()).join("rectifier/rectifier.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let mut solver = Solver::new(circuit, config);

        let tran_analysis = TransientAnalysis {
            time_step: 50e-6, // 50us
            stop_time: 50e-3, // 20ms
        };

        let solution = solver.solve(Analysis::Transient(tran_analysis)).unwrap();
        print_results_to_console(&solution);
        // let transient_solution = solution.clone().into_transient();
    }

    #[test]
    fn test_low_pass_filter_transient() {
        let path = Path::new(&circuits_dir()).join("low_pass_filter/transient.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let mut solver = Solver::new(circuit, config);

        let tran_analysis = TransientAnalysis {
            time_step: 50e-6, // 50us
            stop_time: 20e-3, // 20ms
        };

        let solution = solver.solve(Analysis::Transient(tran_analysis)).unwrap();
        // print_results_to_console(&solution);
        let transient_solution = solution.clone().into_transient();

        let result_t0 = &transient_solution[0];
        assert!((result_t0.get("V(in)").unwrap() - 0.0).abs() < 1e-3);
        assert!((result_t0.get("V(out)").unwrap() - 0.0).abs() < 1e-3);

        // At 2ms, the output should be close to 1V (steady state for a step input)
        // V(out) = 1 - exp(-t/RC)  ≈ 0.8647

        let result_2ms = &transient_solution[42];

        assert!((result_2ms.get("V(out)").unwrap() - 0.8647).abs() < 1e-3);
        assert!((result_2ms.get("time").unwrap() - 2.1e-3).abs() < 1e-6);

        let result_last = transient_solution.last().unwrap();
        assert!((result_last.get("V(out)").unwrap() - 1.0).abs() < 1e-3);

        // print_results_to_console(&solution);
    }

    #[test]
    fn test_high_pass_filter_transient() {
        let path = Path::new(&circuits_dir()).join("high_pass_filter/transient.cir");
        let circuit = krets_parser::parser::parse_circuit_description_file(&path).unwrap();
        let config = SolverConfig::default();
        let mut solver = Solver::new(circuit, config);

        let tran_analysis = TransientAnalysis {
            time_step: 10e-6, // 10us
            stop_time: 2e-3,  // 2ms
        };

        let solution = solver.solve(Analysis::Transient(tran_analysis)).unwrap();
        print_results_to_console(&solution);
        let transient_solution = solution.clone().into_transient();

        let result_t0 = &transient_solution[0];
        assert!((result_t0.get("V(in)").unwrap() - 0.0).abs() < 1e-3);
        assert!((result_t0.get("V(out)").unwrap() - 0.0).abs() < 1e-3);

        let result_last = transient_solution.last().unwrap();
        assert!((result_last.get("V(out)").unwrap() - 1.0).abs() < 1e-3);

        // print_results_to_console(&solution);
    }
}
