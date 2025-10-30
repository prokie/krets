use crate::prelude::*;

pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod nmosfet;
pub mod resistor;
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
            Element::Diode(_) | Element::BJT(_) | Element::NMOSFET(_) => false,
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
pub trait Stampable {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;
    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;
    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;
    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;

    /// Default implementation for resistive elements. Their transient stamp is the same as their DC stamp.
    fn add_conductance_matrix_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        self.add_conductance_matrix_dc_stamp(index_map, solution_map)
    }

    /// Default implementation for resistive elements.
    fn add_excitation_vector_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        self.add_excitation_vector_dc_stamp(index_map, solution_map)
    }
}

impl Identifiable for Element {
    fn identifier(&self) -> String {
        dispatch!(self, identifier())
    }
}

impl Stampable for Element {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            add_conductance_matrix_dc_stamp(index_map, solution_map)
        )
    }
    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            add_excitation_vector_dc_stamp(index_map, solution_map)
        )
    }
    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        dispatch!(
            self,
            add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
        )
    }
    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        dispatch!(
            self,
            add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
        )
    }
    fn add_conductance_matrix_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            add_conductance_matrix_transient_stamp(
                index_map,
                solution_map,
                prev_solution,
                time_step
            )
        )
    }
    fn add_excitation_vector_transient_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        dispatch!(
            self,
            add_excitation_vector_transient_stamp(
                index_map,
                solution_map,
                prev_solution,
                time_step
            )
        )
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
