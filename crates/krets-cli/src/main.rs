use clap::Parser;
use krets_gui::run_gui;
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

    /// Whether to launch the GUI.
    #[arg(short, long, default_value_t = true)]
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

    // decide output file path: always write result.parquet next to the krets file
    let output_path_buf = krets_parent.join("result.parquet");
    let output_file_str = output_path_buf.to_string_lossy().into_owned();

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
    // print_results_to_console(&result);

    match &result {
        AnalysisResult::Op(op_solution) => {
            write_op_results_to_parquet(op_solution, &output_file_str).unwrap_or_else(|e| {
                eprintln!("Error writing OP results to Parquet: {e}");
                std::process::exit(1);
            });
        }
        AnalysisResult::Dc(dc_solution) => {
            write_dc_results_to_parquet(dc_solution, &output_file_str).unwrap_or_else(|e| {
                eprintln!("Error writing DC results to Parquet: {e}");
                std::process::exit(1);
            });
        }
        AnalysisResult::Ac(_) => {
            eprintln!("AC results Parquet export not implemented yet.");
        }
        AnalysisResult::Transient(tran_solution) => {
            write_tran_results_to_parquet(tran_solution, &output_file_str).unwrap_or_else(|e| {
                eprintln!("Error writing Transient results to Parquet: {e}");
                std::process::exit(1);
            });
        }
    }

    // 7. Optionally launch the GUI.
    if args.gui {
        let _ = run_gui(
            circuit_path_resolved
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."))
                .to_path_buf(),
            Some(output_path_buf.clone()),
        );
    }
}
