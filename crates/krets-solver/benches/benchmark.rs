use criterion::{Criterion, criterion_group, criterion_main};
use krets_parser::analyses::{AcAnalysis, AcSweep, Analysis, DcAnalysis, TransientAnalysis};
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

fn benchmark_dc_voltage_divider(c: &mut Criterion) {
    let path = Path::new("../../circuits/voltage_divider/voltage_divider.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let dc_analysis = DcAnalysis {
        element: "V1".to_string(),
        start: 0.0,
        stop: 1.0,
        step_size: 0.01, // 100 steps
    };
    let analysis = Analysis::Dc(dc_analysis);

    c.bench_function("dc_voltage_divider_100_steps", |b| {
        b.iter(|| {
            let mut solver = Solver::new(circuit.clone(), config.clone());
            let solution = solver.solve(analysis.clone());
            let _ = black_box(solution);
        })
    });
}

fn benchmark_ac_low_pass_filter(c: &mut Criterion) {
    let path = Path::new("../../circuits/low_pass_filter/low_pass_filter.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let ac_analysis = AcAnalysis {
        sweep: AcSweep::Linear { total_points: 100 },
        fstart: 1.0,
        fstop: 1000.0,
    };
    let analysis = Analysis::Ac(ac_analysis);

    c.bench_function("ac_low_pass_filter_100_points", |b| {
        b.iter(|| {
            let mut solver = Solver::new(circuit.clone(), config.clone());
            let solution = solver.solve(analysis.clone());
            let _ = black_box(solution);
        })
    });
}

fn benchmark_tran_dual_rc_ladder(c: &mut Criterion) {
    let path = Path::new("../../circuits/dual_rc_ladder/dual_rc_ladder.cir");
    let circuit = krets_parser::parser::parse_circuit_description_file(path).unwrap();
    let config = SolverConfig::default();
    let tran_analysis = TransientAnalysis {
        time_step: 50e-6, // 50us
        stop_time: 50e-3, // 50ms (1000 steps)
    };
    let analysis = Analysis::Transient(tran_analysis);

    c.bench_function("tran_dual_rc_ladder_1000_steps", |b| {
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
    benchmark_resistor_ladder_5000,
    benchmark_dc_voltage_divider,
    benchmark_ac_low_pass_filter,
    benchmark_tran_dual_rc_ladder
);
criterion_main!(benches);
