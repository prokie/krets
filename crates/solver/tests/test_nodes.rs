#[cfg(test)]
mod tests {
    use parser::parse_netlist;

    #[test]
    fn test_parse_complex_netlist() {
        let netlist = "
    Title
    R1 1 2 100
    R2 2 3 200
    C1 1 3 10u
    L1 3 0 1m
    V1 1 0 10
    I1 2 0 5
    ";

        let netlist = parse_netlist(netlist).unwrap();
        let solver = solver::Solver::new(netlist);

        assert!(solver.nodes.len() == 4);

        assert!(solver.nodes.contains(&"0".to_string()));
        assert!(solver.nodes.contains(&"1".to_string()));
        assert!(solver.nodes.contains(&"2".to_string()));
        assert!(solver.nodes.contains(&"3".to_string()));
    }

    #[test]
    fn test_generate_matrix_g_case_1() {
        let netlist = "
    Title
    V1 1 2 32
    R1 1 0 2
    R2 2 3 4
    R3 2 0 8
    V2 3 0 20
";

        let netlist = parse_netlist(netlist).unwrap();
        let solver = solver::Solver::new(netlist);

        let matrix_g = solver.generate_matrix_g();
        dbg!(matrix_g.clone());

        assert!((matrix_g[(0, 0)] - 0.5).abs() < 1e-9);
        assert!((matrix_g[(0, 1)] - 0.0).abs() < 1e-9);
        assert!((matrix_g[(0, 2)] - 0.0).abs() < 1e-9);

        assert!((matrix_g[(1, 0)] - 0.0).abs() < 1e-9);
        assert!((matrix_g[(1, 1)] - 0.375).abs() < 1e-9);
        assert!((matrix_g[(1, 2)] - -0.25).abs() < 1e-9);

        assert!((matrix_g[(2, 0)] - 0.0).abs() < 1e-9);
        assert!((matrix_g[(2, 1)] - -0.25).abs() < 1e-9);
        assert!((matrix_g[(2, 2)] - 0.25).abs() < 1e-9);
    }

    #[test]
    fn test_generate_matrix_g_case_2() {
        let netlist = "
    Title
    V1 1 2 5
    I1 1 0 2

    R1 0 1 1k
    R2 1 2 2k
    R3 2 0 3k
";

        let netlist = parse_netlist(netlist).unwrap();
        let solver = solver::Solver::new(netlist);

        let matrix_g = solver.generate_matrix_g();

        assert!((matrix_g[(0, 0)] - (1.0 / 1000.0 + 1.0 / 2000.0)).abs() < 1e-9);
        assert!((matrix_g[(0, 1)] - -1.0 / 2000.0).abs() < 1e-9);

        assert!((matrix_g[(1, 0)] - -1.0 / 2000.0).abs() < 1e-9);
        assert!((matrix_g[(1, 1)] - (1. / 2000.0 + 1. / 3000.0)).abs() < 1e-9);
    }
}
