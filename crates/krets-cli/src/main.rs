use clap::Parser;
use krets_solver::solver::Solver;

/// Krets is a circuit simulator.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The circuit to simulate.
    #[arg()]
    circuit: String,
}

fn main() {
    let args = Args::parse();

    let circuit_path = std::path::Path::new(&args.circuit);
    let circuit = krets_parser::parser::parse_circuit_description_file(circuit_path).unwrap();

    let solver = Solver::new(circuit);
    let solution = solver.solve_op();

    println!("{:?}", solution);
}
