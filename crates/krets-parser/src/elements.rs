use crate::prelude::*;
use nom::{Parser, branch::alt};
pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod nmosfet;
pub mod resistor;
pub mod subcircuit;
pub mod voltage_source;
/// Represents any component that can be included in a circuit simulation.
#[derive(Debug, Clone)]
pub enum Element {
    VoltageSource(voltage_source::VoltageSource),
    CurrentSource(current_source::CurrentSource),
    Resistor(resistor::Resistor),
    Capacitor(capacitor::Capacitor),
    Inductor(inductor::Inductor),
    Diode(diode::Diode),
    BJT(bjt::BJT),
    NMOSFET(nmosfet::NMOSFET),
    SubcktInstance(subcircuit::SubcircuitInstance),
}

pub fn parse_element(input: &str) -> Result<Element> {
    // Remove comments starting with '%'
    let input = input.split('%').next().unwrap_or("").trim();

    let (_, element) = alt((
        map(parse_resistor, Element::Resistor),
        map(parse_capacitor, Element::Capacitor),
        map(parse_inductor, Element::Inductor),
        map(parse_voltage_source, Element::VoltageSource),
        map(parse_current_source, Element::CurrentSource),
        map(parse_diode, Element::Diode),
        map(parse_bjt, Element::BJT),
        map(parse_nmosfet, Element::NMOSFET),
        map(parse_subckt_instance, Element::SubcktInstance),
    ))
    .parse(input)
    .map_err(|e| {
        Error::Unexpected(format!(
            "Failed to parse element from input '{}': parser error: {:?}",
            input, e
        ))
    })?;

    Ok(element)
}

/// A macro to forward a method call to the correct inner element struct.
/// This reduces boilerplate code for the `Element` enum wrappers.
macro_rules! dispatch {
    ($self:expr, $method:ident($($args:expr),*)) => {
        match $self {
            Element::VoltageSource(e) => e.$method($($args),*),
            Element::CurrentSource(e) => e.$method($($args),*),
            Element::Resistor(e) => e.$method($($args),*),
            Element::Capacitor(e) => e.$method($($args),*),
            Element::Inductor(e) => e.$method($($args),*),
            Element::Diode(e) => e.$method($($args),*),
            Element::BJT(e) => e.$method($($args),*),
            Element::NMOSFET(e) => e.$method($($args),*),
            Element::SubcktInstance(e) => e.$method($($args),*),
        }
    };
}

impl From<voltage_source::VoltageSource> for Element {
    fn from(item: voltage_source::VoltageSource) -> Self {
        Element::VoltageSource(item)
    }
}
impl From<current_source::CurrentSource> for Element {
    fn from(item: current_source::CurrentSource) -> Self {
        Element::CurrentSource(item)
    }
}
impl From<resistor::Resistor> for Element {
    fn from(item: resistor::Resistor) -> Self {
        Element::Resistor(item)
    }
}
impl From<capacitor::Capacitor> for Element {
    fn from(item: capacitor::Capacitor) -> Self {
        Element::Capacitor(item)
    }
}
impl From<inductor::Inductor> for Element {
    fn from(item: inductor::Inductor) -> Self {
        Element::Inductor(item)
    }
}
impl From<diode::Diode> for Element {
    fn from(item: diode::Diode) -> Self {
        Element::Diode(item)
    }
}
impl From<bjt::BJT> for Element {
    fn from(item: bjt::BJT) -> Self {
        Element::BJT(item)
    }
}
impl From<nmosfet::NMOSFET> for Element {
    fn from(item: nmosfet::NMOSFET) -> Self {
        Element::NMOSFET(item)
    }
}

impl Element {
    /// Retrieves the nodes associated with the element.
    pub fn nodes(&self) -> Vec<&str> {
        match self {
            Element::VoltageSource(v) => vec![&v.plus, &v.minus],
            Element::CurrentSource(i) => vec![&i.plus, &i.minus],
            Element::Resistor(r) => vec![&r.plus, &r.minus],
            Element::Capacitor(c) => vec![&c.plus, &c.minus],
            Element::Inductor(l) => vec![&l.plus, &l.minus],
            Element::Diode(d) => vec![&d.plus, &d.minus],
            Element::BJT(b) => vec![&b.collector, &b.emitter, &b.base],
            Element::NMOSFET(m) => vec![&m.drain, &m.gate, &m.source],
            Element::SubcktInstance(s) => s.nodes.iter().map(String::as_str).collect(),
        }
    }

    /// Determines if the element requires a dedicated branch current (Group 2) in MNA.
    pub fn is_g2(&self) -> bool {
        match self {
            // Voltage sources and inductors are always group 2.
            Element::VoltageSource(_) => true,
            Element::Inductor(_) => true,
            // The parser determines if these are Group 2.
            Element::Resistor(e) => e.g2,
            Element::Capacitor(e) => e.g2,
            Element::CurrentSource(_) => true,
            // Non-linear elements are linearized into Group 1 companion models.
            Element::Diode(_)
            | Element::BJT(_)
            | Element::NMOSFET(_)
            | Element::SubcktInstance(_) => false,
        }
    }

    /// Checks if the element is non-linear.
    pub fn is_nonlinear(&self) -> bool {
        matches!(
            self,
            Element::Diode(_) | Element::BJT(_) | Element::NMOSFET(_)
        )
    }
}

/// A trait for elements that have a unique string identifier.
pub trait Identifiable {
    fn identifier(&self) -> String;
}

/// A trait for elements that can contribute to the MNA matrices.
/// A trait for elements that can contribute their "stamp" to the Modified Nodal Analysis (MNA) matrices.
///
/// Implementors of this trait provide methods to add their contributions to the conductance matrix and excitation vector
/// for DC, AC, and transient analyses. These methods are called during circuit simulation to assemble the system equations.
///
/// The default implementations for transient stamps assume resistive behavior, using the DC stamp.
pub trait Stampable {
    /// Adds the DC conductance matrix stamp for this element.
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to matrix indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the conductance matrix.
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;

    /// Adds the DC excitation vector stamp for this element.
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to vector indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the excitation vector.
    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;

    /// Adds the AC conductance matrix stamp for this element at a given frequency.
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to matrix indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    /// * `frequency` - The AC analysis frequency.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the AC conductance matrix.
    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;

    /// Adds the AC excitation vector stamp for this element at a given frequency.
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to vector indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    /// * `frequency` - The AC analysis frequency.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the AC excitation vector.
    fn stamp_excitation_vector_ac(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;

    /// Adds the transient conductance matrix stamp for this element.
    ///
    /// By default, uses the DC stamp (appropriate for resistive elements).
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to matrix indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    /// * `prev_solution` - Solution values from the previous time step.
    /// * `time_step` - The simulation time step.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the transient conductance matrix.
    fn stamp_conductance_matrix_transient(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        self.stamp_conductance_matrix_dc(index_map, solution_map)
    }

    /// Adds the transient excitation vector stamp for this element.
    ///
    /// By default, uses the DC excitation vector stamp (appropriate for resistive elements).
    ///
    /// # Arguments
    /// * `index_map` - Maps node/branch identifiers to vector indices.
    /// * `solution_map` - Current solution values for nodes/branches.
    /// * `prev_solution` - Solution values from the previous time step.
    /// * `time_step` - The simulation time step.
    ///
    /// # Returns
    /// A vector of triplets representing non-zero entries in the transient excitation vector.
    fn stamp_excitation_vector_transient(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        self.stamp_excitation_vector_dc(index_map, solution_map)
    }
}

impl Identifiable for Element {
    fn identifier(&self) -> String {
        dispatch!(self, identifier())
    }
}

impl Stampable for Element {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(self, stamp_conductance_matrix_dc(index_map, solution_map))
    }
    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(self, stamp_excitation_vector_dc(index_map, solution_map))
    }
    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        dispatch!(
            self,
            stamp_conductance_matrix_ac(index_map, solution_map, frequency)
        )
    }
    fn stamp_excitation_vector_ac(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        dispatch!(
            self,
            stamp_excitation_vector_ac(index_map, solution_map, frequency)
        )
    }
    fn stamp_conductance_matrix_transient(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            stamp_conductance_matrix_transient(index_map, solution_map, prev_solution, time_step)
        )
    }
    fn stamp_excitation_vector_transient(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            stamp_excitation_vector_transient(index_map, solution_map, prev_solution, time_step)
        )
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
