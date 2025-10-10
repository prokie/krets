use faer::{c64, sparse::Triplet};
use std::collections::HashMap;

pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod mosfet;
pub mod resistor;
pub mod voltage_source;

/// Represents any component that can be included in a circuit simulation.
///
/// This enum acts as a wrapper around the specific struct for each element type,
/// allowing for a heterogeneous collection of circuit components.
#[derive(Debug, Clone)]
pub enum Element {
    /// A voltage source element.
    VoltageSource(voltage_source::VoltageSource),
    /// A current source element.
    CurrentSource(current_source::CurrentSource),
    /// A resistor element.
    Resistor(resistor::Resistor),
    /// A capacitor element.
    Capacitor(capacitor::Capacitor),
    /// An inductor element.
    Inductor(inductor::Inductor),
    /// A diode element.
    Diode(diode::Diode),
    /// Bipolar Junction Transistor (BJT) element.
    BJT(bjt::BJT),
    /// Metal-Oxide-Semiconductor Field-Effect Transistor (MOSFET) element.
    MOSFET(mosfet::MOSFET),
}

// Manually implement the `From` trait for each element variant.
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
impl From<mosfet::MOSFET> for Element {
    fn from(item: mosfet::MOSFET) -> Self {
        Element::MOSFET(item)
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
            Element::MOSFET(m) => vec![&m.drain, &m.gate, &m.source],
        }
    }

    /// Determines if the element requires a dedicated branch current (Group 2) in MNA.
    pub fn is_g2(&self) -> bool {
        match self {
            // Voltage sources and inductors always require a branch current in MNA.
            Element::VoltageSource(_) => true,
            Element::Inductor(_) => true,
            // Non-linear elements are linearized into Group 1 companion models.
            Element::Diode(_) => false,
            Element::BJT(_) => false,
            Element::MOSFET(_) => false,
            // For these elements, group membership depends on the netlist definition.
            Element::CurrentSource(_) => true,
            Element::Resistor(_) => true,
            Element::Capacitor(e) => e.g2,
        }
    }

    /// Checks if the element is non-linear.
    pub fn is_nonlinear(&self) -> bool {
        matches!(
            self,
            Element::Diode(_) | Element::BJT(_) | Element::MOSFET(_)
        )
    }
}

/// A trait for elements that have a unique string identifier.
pub trait Identifiable {
    /// Returns the SPICE-like identifier for the element (e.g., "R1", "V_SOURCE").
    fn identifier(&self) -> String;
}

/// A trait for elements that can contribute to the MNA matrices.
///
/// This unified trait handles contributions for both DC and AC analyses, providing a single,
/// consistent interface for all stampable circuit components.
pub trait Stampable {
    /// Contributes to the DC conductance matrix (the `G` matrix in `Gx=z`).
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;

    /// Contributes to the DC excitation vector (the `z` vector in `Gx=z`).
    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>>;

    /// Contributes to the AC conductance matrix (the `Y` matrix in `Yx=z`).
    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;

    /// Contributes to the AC excitation vector (the `z` vector in `Yx=z`).
    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>>;
}

impl Identifiable for Element {
    fn identifier(&self) -> String {
        match self {
            Element::VoltageSource(e) => e.identifier(),
            Element::CurrentSource(e) => e.identifier(),
            Element::Resistor(e) => e.identifier(),
            Element::Capacitor(e) => e.identifier(),
            Element::Inductor(e) => e.identifier(),
            Element::Diode(e) => e.identifier(),
            Element::BJT(e) => e.identifier(),
            Element::MOSFET(e) => e.identifier(),
        }
    }
}

impl Stampable for Element {
    fn add_conductance_matrix_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        match self {
            Element::VoltageSource(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::CurrentSource(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::Resistor(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::Capacitor(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::Inductor(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::Diode(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::BJT(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
            Element::MOSFET(e) => e.add_conductance_matrix_dc_stamp(index_map, solution_map),
        }
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        match self {
            Element::VoltageSource(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::CurrentSource(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Resistor(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Capacitor(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Inductor(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Diode(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::BJT(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
            Element::MOSFET(e) => {
                e.add_conductance_matrix_ac_stamp(index_map, solution_map, frequency)
            }
        }
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        match self {
            Element::VoltageSource(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::CurrentSource(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::Resistor(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::Capacitor(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::Inductor(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::Diode(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::BJT(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
            Element::MOSFET(e) => e.add_excitation_vector_dc_stamp(index_map, solution_map),
        }
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        match self {
            Element::VoltageSource(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::CurrentSource(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Resistor(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Capacitor(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Inductor(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::Diode(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
            Element::BJT(e) => e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency),
            Element::MOSFET(e) => {
                e.add_excitation_vector_ac_stamp(index_map, solution_map, frequency)
            }
        }
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
