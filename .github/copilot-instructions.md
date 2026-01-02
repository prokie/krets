# Krets Copilot Instructions

Krets is a SPICE circuit simulator written in Rust, designed for learning and educational purposes.

## Architecture Overview

The project uses a **modular monorepo architecture** with five Rust crates:

- **krets-parser**: Parses SPICE netlists and analysis specifications from TOML files
- **krets-solver**: Core simulation engine implementing Modified Nodal Analysis (MNA); dispatches to submodules for OP, DC sweep, AC, and transient analyses
- **krets-result**: Outputs results to Parquet format using Polars
- **krets-cli**: Main entry point; orchestrates the workflow: parse → solve → output results
- **krets-gui**: (Egui-based) graphical interface (emerging; not all analyses fully integrated)

### Data Flow

```
Circuit file (.cir) + Analysis spec (krets.toml)
         ↓
[krets-parser: parse_circuit_description_file()]
         ↓
Circuit struct (elements, nodes, index_map)
         ↓
[krets-solver: Solver.solve(analysis)]
         ↓
AnalysisResult enum (Op, Dc, Ac, Transient)
         ↓
[krets-result: write_*_results_to_parquet()]
         ↓
result.parquet (next to krets.toml)
```

## Key Conventions

### Element & Stamp System
- All circuit elements implement a **dispatch macro** pattern (in [elements.rs](crates/krets-parser/src/elements.rs) and [stampable.rs](crates/krets-solver/src/stampable.rs))
- New elements must implement the **`Stampable` trait** in [krets-solver](crates/krets-solver/src/stampable.rs), providing DC/AC/transient stamps (conductance matrix & excitation vector contributions)
- Submodules: [voltage_source.rs](crates/krets-parser/src/elements/voltage_source.rs), [resistor.rs](crates/krets-parser/src/elements/resistor.rs), [capacitor.rs](crates/krets-parser/src/elements/capacitor.rs), [inductor.rs](crates/krets-parser/src/elements/inductor.rs), [diode.rs](crates/krets-parser/src/elements/diode.rs), [bjt.rs](crates/krets-parser/src/elements/bjt.rs), [nmosfet.rs](crates/krets-parser/src/elements/nmosfet.rs)

### Parser Conventions
- **Case-insensitive** netlist parsing with whitespace normalization
- Node names are **string identifiers** (including integer nodes like "0" for ground)
- Each element line is fully independent; order in netlist is irrelevant
- Comments: lines/text after `%` or `*` are stripped
- [parse_circuit_description()](crates/krets-parser/src/parser.rs) creates `index_map` (node → matrix index) and validates nodes

### Solver Architecture
- **Solver::solve()** dispatches to submodules: [op.rs](crates/krets-solver/src/solver/op.rs), [dc.rs](crates/krets-solver/src/solver/dc.rs), [ac.rs](crates/krets-solver/src/solver/ac.rs), [transient.rs](crates/krets-solver/src/solver/transient.rs)
- DC/transient use **Newton-Raphson iteration** for nonlinear convergence (see `convergence_check()` in [solver.rs](crates/krets-solver/src/solver.rs))
- AC analysis uses **complex numbers (c64)** via Faer library
- MNA matrix construction uses **Triplet format** aggregation via `sum_triplets()`

### Test Patterns
- Tests use `manifest_dir()` / `circuits_dir()` helpers to dynamically locate circuit files relative to workspace root
- Tests in [test_op.rs](crates/krets-solver/tests/test_op.rs) instantiate `SolverConfig::default()` and call `solver.solve(analysis)?.into_op/dc/ac/transient()`
- Circuit test files live in [circuits/](circuits/) directory (e.g., [circuits/basic_001/basic_001.cir](circuits/basic_001/basic_001.cir))
- Assertion tolerance: typically `abs() < 1e-3` for voltage/current validation

## Critical Build & Workflow Commands

```bash
# Build all crates
cargo build --release

# Run parser tests
cargo test -p krets-parser

# Run solver tests (includes circuit simulations)
cargo test -p krets-solver

# Run a simulation from CLI
cargo run -p krets-cli -- circuits/low_pass_filter/ac/krets.toml

# Check for lints (Clippy configured permissively in workspace Cargo.toml)
cargo clippy --all
```

## Important Implementation Details

- **Error Handling**: Uses `thiserror` crate with custom `Error` enum in [error.rs](crates/krets-parser/src/error.rs)
- **Logging**: `tracing` / `log` crate; CLI sets log level via `--log-level` flag
- **Sparse Matrices**: Uses Faer's triplet-based sparse matrix API; only non-zero entries are stored
- **AC Analysis**: Results use complex numbers (`c64`); magnitude/phase extraction is the responsibility of result writers
- **Transient Solver**: Uses backward Euler (implicit) time-stepping; capacitor/inductor memory updates required at each step
- **Config File**: [krets.toml](crates/krets-parser/src/analyses.rs) specifies `circuit_path` (relative to config file) and `[analysis.*]` section with variant-specific parameters

## Common Pitfalls

- **Relative Paths**: CLI resolves circuit paths relative to the krets.toml file location, not the current working directory
- **Output Location**: `result.parquet` is always written to the directory containing the krets.toml file
- **Nonlinear Elements**: Diodes, BJTs, MOSFETs require Newton-Raphson; ensure `stamp_conductance_matrix_dc()` + `stamp_excitation_vector_dc()` work together for proper linearization
- **Subcircuit Expansion**: Currently parsed but not fully simulated; subcircuits are instantiated but may require additional integration
- **AC Frequency Arrays**: Build using decade/octave/linear variations; validate frequency array construction in [analyses.rs](crates/krets-parser/src/analyses.rs) before use

## File Organization Reference

- **Parser**: `crates/krets-parser/src/` — netlist → Circuit struct
- **Solver**: `crates/krets-solver/src/solver/` — analysis-specific logic
- **Elements**: `crates/krets-parser/src/elements/` (parser) + `crates/krets-solver/src/stampable.rs` (solver)
- **Tests**: `crates/*/tests/` — integration tests with real circuits
- **Example Circuits**: `circuits/` — reference netlists for testing and validation
