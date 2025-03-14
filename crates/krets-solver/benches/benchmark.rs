use criterion::{Criterion, black_box, criterion_group, criterion_main};
use krets_solver::solver::Solver;
use std::path::Path;

fn benchmark_resistor_ladder_500(c: &mut Criterion) {
    let path = Path::new("../../circuits/resistor_ladder_500/resistor_ladder_500.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    // let solver = Solver::new(circuit);

    c.bench_function("resistor_ladder_500", |b| {
        b.iter(|| {
            let solver = Solver::new(circuit.clone());
            let solution = solver.solve_op();
            black_box(solution);
        })
    });
}

fn benchmark_resistor_ladder_1000(c: &mut Criterion) {
    let path = Path::new("../../circuits/resistor_ladder_1000/resistor_ladder_1000.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();

    c.bench_function("resistor_ladder_1000", |b| {
        b.iter(|| {
            let solver = Solver::new(circuit.clone());
            let solution = solver.solve_op();
            black_box(solution);
        })
    });
}

criterion_group!(
    benches,
    benchmark_resistor_ladder_500,
    benchmark_resistor_ladder_1000
);
criterion_main!(benches);
