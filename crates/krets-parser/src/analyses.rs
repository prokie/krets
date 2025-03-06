pub enum Analyses {
    // DC analysis
    DC,

    // AC Small-Signal Analysis
    AC,

    // Transient Analysis
    Transient,

    // Operating Point
    OP,
}

pub struct DcAnalysis {
    pub element: String,
    pub start: f64,
    pub stop: f64,
    pub step_size: f64,
}
