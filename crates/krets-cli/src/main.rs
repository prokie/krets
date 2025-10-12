use clap::Parser;
use krets_parser::analyses::{Analysis, AnalysisResult};
use krets_result::{write_dc_results_to_parquet, write_op_results_to_parquet};
use krets_solver::{config::SolverConfig, solver::Solver};

use std::path::Path;

/// Krets is a SPICE-like circuit simulator written in Rust.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the circuit netlist file to simulate.
    #[arg()]
    circuit_file: String,

    /// Optional path to save the results to a Parquet file.
    #[arg(short, long)]
    output: Option<String>,
}

fn main() {
    let args = Args::parse();
    let circuit_path = Path::new(&args.circuit_file);

    // 1. Parse the circuit description file with robust error handling.
    let circuit = match krets_parser::parser::parse_circuit_description_file(circuit_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error parsing circuit file '{}': {}", args.circuit_file, e);
            std::process::exit(1);
        }
    };

    // 2. Create a default solver configuration.
    let config = SolverConfig::default();

    // 3. Instantiate the solver.
    let mut solver = Solver::new(circuit, config);

    // 4. Define the analysis to run (TODO: This should be driven by CLI args).
    let analysis = Analysis::Op;

    println!(
        "Running {:?} analysis on '{}'...",
        analysis, args.circuit_file
    );

    // 5. Run the solver and handle the result.
    match solver.solve(analysis) {
        Ok(result) => {
            println!("\n--- Analysis successful ---");

            if let Some(filename) = args.output {
                // If an output file is specified, write to Parquet.
                let write_result = match &result {
                    AnalysisResult::Op(data) => write_op_results_to_parquet(data, &filename),
                    AnalysisResult::Dc(data) => write_dc_results_to_parquet(data, &filename),
                    AnalysisResult::Ac(_) => {
                        println!("Parquet export for AC analysis is not yet supported.");
                        Ok(())
                    }
                    AnalysisResult::Transient(_) => {
                        println!("Parquet export for Transient analysis is not yet supported.");
                        Ok(())
                    }
                };
                if let Err(e) = write_result {
                    eprintln!("Error writing to Parquet file: {}", e);
                }
            } else {
                // Otherwise, print to console.
                print_results_to_console(&result);
            }
        }
        Err(e) => {
            eprintln!("\n--- Error during simulation ---");
            eprintln!("{}", e);
            std::process::exit(1);
        }
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
                println!("{:<15} | {:>19.6e} | {:>19.6e}", node, mag, phase_deg);
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
