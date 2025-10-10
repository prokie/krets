use criterion::{Criterion, criterion_group, criterion_main};
use krets_parser::analyses::Analysis;
use krets_solver::{config::SolverConfig, solver::Solver};
use std::hint::black_box;
use std::path::Path;

fn benchmark_resistor_ladder_500(c: &mut Criterion) {
    let path = Path::new("../../circuits/resistor_ladder_500/resistor_ladder_500.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let analysis = Analysis::Op;

    c.bench_function("resistor_ladder_500", |b| {
        b.iter(|| {
            // Re-initialize the solver in each iteration to benchmark the full setup and solve.
            let mut solver = Solver::new(circuit.clone(), config.clone());
            let solution = solver.solve(analysis.clone());
            let _ = black_box(solution);
        })
    });
}

fn benchmark_resistor_ladder_1000(c: &mut Criterion) {
    let path = Path::new("../../circuits/resistor_ladder_1000/resistor_ladder_1000.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let analysis = Analysis::Op;

    c.bench_function("resistor_ladder_1000", |b| {
        b.iter(|| {
            let mut solver = Solver::new(circuit.clone(), config.clone());
            let solution = solver.solve(analysis.clone());
            let _ = black_box(solution);
        })
    });
}

fn benchmark_resistor_ladder_5000(c: &mut Criterion) {
    let path = Path::new("../../circuits/resistor_ladder_5000/resistor_ladder_5000.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let analysis = Analysis::Op;

    c.bench_function("resistor_ladder_5000", |b| {
        b.iter(|| {
            let mut solver = Solver::new(circuit.clone(), config.clone());
            let solution = solver.solve(analysis.clone());
            let _ = black_box(solution);
        })
    });
}

criterion_group!(
    benches,
    benchmark_resistor_ladder_500,
    benchmark_resistor_ladder_1000,
    benchmark_resistor_ladder_5000
);
criterion_main!(benches);
