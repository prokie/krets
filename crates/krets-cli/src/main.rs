use clap::Parser;
use krets_parser::analyses::{AnalysisResult, AnalysisSpec};
use krets_result::{
    write_dc_results_to_parquet, write_op_results_to_parquet, write_tran_results_to_parquet,
};
use krets_solver::{config::SolverConfig, solver::Solver};

/// Krets is a SPICE-like circuit simulator written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the krets file to simulate.
    #[arg()]
    krets_file: String,

    /// Optional path to save the results to a Parquet file.
    #[arg(short, long)]
    output: Option<String>,

    /// Whether to launch the GUI.
    #[arg(short, long, default_value_t = false)]
    gui: bool,
}

fn main() {
    let args = Args::parse();

    let krets_spec = AnalysisSpec::from_file(&args.krets_file).unwrap_or_else(|e| {
        eprintln!("Error reading krets spec from '{}': {}", args.krets_file, e);
        std::process::exit(1);
    });

    // Resolve circuit path: prefer path relative to the krets spec file, otherwise accept an absolute path.
    let krets_file_path = std::path::Path::new(&args.krets_file);
    let krets_parent = krets_file_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));

    // First try the path interpreted relative to the krets file.
    let rel_candidate = krets_parent.join(&krets_spec.circuit_path);
    let circuit_path_resolved = if rel_candidate.exists() {
        rel_candidate
    } else if krets_spec.circuit_path.is_absolute() && krets_spec.circuit_path.exists() {
        // Fallback: if the given path is absolute and exists, use it.
        krets_spec.circuit_path.clone()
    } else {
        eprintln!(
            "Circuit file not found.\nTried:\n  - relative to krets file: {}\n  - as given (absolute or relative to cwd): {}\n\nProvide a path that exists either relative to the krets file or as an absolute path.",
            rel_candidate.display(),
            krets_spec.circuit_path.display()
        );
        std::process::exit(1);
    };

    // 1. Parse the circuit description file with robust error handling.
    let circuit = match krets_parser::parser::parse_circuit_description_file(&circuit_path_resolved)
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "Error parsing circuit file '{}': {}",
                circuit_path_resolved.display(),
                e
            );
            std::process::exit(1);
        }
    };

    // 2. Create a default solver configuration.
    let config = SolverConfig::default();

    // 3. Instantiate the solver.
    let mut solver = Solver::new(circuit, config);

    let analysis = krets_spec.analysis;

    println!(
        "Running {:?} analysis on '{}'...",
        analysis,
        krets_spec.circuit_path.display()
    );

    // 4. Run the specified analysis.
    let result = solver.solve(analysis).unwrap_or_else(|e| {
        eprintln!("Error during analysis: {e}");
        std::process::exit(1);
    });

    // 5. Print results to console.
    print_results_to_console(&result);

    // 6. Optionally write results to Parquet file.
    if let Some(output_path) = args.output {
        match &result {
            AnalysisResult::Op(op_solution) => {
                write_op_results_to_parquet(op_solution, &output_path).unwrap_or_else(|e| {
                    eprintln!("Error writing OP results to Parquet: {e}");
                    std::process::exit(1);
                });
            }
            AnalysisResult::Dc(dc_solution) => {
                write_dc_results_to_parquet(dc_solution, &output_path).unwrap_or_else(|e| {
                    eprintln!("Error writing DC results to Parquet: {e}");
                    std::process::exit(1);
                });
            }
            AnalysisResult::Ac(_) => {
                eprintln!("AC results Parquet export not implemented yet.");
            }
            AnalysisResult::Transient(tran_solution) => {
                write_tran_results_to_parquet(tran_solution, &output_path).unwrap_or_else(|e| {
                    eprintln!("Error writing Transient results to Parquet: {e}");
                    std::process::exit(1);
                });
            }
        }
        println!("Results written to '{output_path}'.");
    }
}

/// Prints the analysis results to the console in a human-readable format.
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
                println!("{node_or_branch:<15} | {value:>14.6e} {unit}");
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
                print!("{header:<18}");
            }
            println!();
            println!("{:-<width$}", "", width = headers.len() * 18);

            // Print rows
            for step_result in dc_solution {
                for header in &headers {
                    if let Some(value) = step_result.get(*header) {
                        print!("{value:<18.6e}");
                    } else {
                        print!("{:<18}", "N/A");
                    }
                }
                println!();
            }
        }
        AnalysisResult::Ac(ac_solution) => {
            let mut sorted_results: Vec<_> = ac_solution.iter().collect();
            sorted_results.sort_by_key(|(k, _)| *k);

            println!(
                "{:<15} | {:<20} | {:<20}",
                "Node/Branch", "Magnitude", "Phase (deg)"
            );
            println!("{:-<15}-+-{:-<20}-+-{:-<20}", "", "", "");

            for (node, value) in sorted_results {
                let (mag, phase_deg) = (value.norm(), value.arg().to_degrees());
                println!("{node:<15} | {mag:>19.6e} | {phase_deg:>19.6e}");
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
                print!("{header:<18}");
            }
            println!();
            println!("{:-<width$}", "", width = headers.len() * 18);

            // Print rows
            for step_result in tran_solution {
                for header in &headers {
                    if let Some(value) = step_result.get(*header) {
                        print!("{value:<18.6e}");
                    } else {
                        print!("{:<18}", "N/A");
                    }
                }
                println!();
            }
        }
    }
}
