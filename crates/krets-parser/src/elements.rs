use krets_matrix::mna_matrix::MnaMatrix;

pub mod bjt;
pub mod capacitor;
pub mod current_source;
pub mod diode;
pub mod inductor;
pub mod mosfet;
pub mod resistor;
pub mod voltage_source;

/// Represents a circuit element.
///
/// # Element Groups
/// Definition 2.6. (Element Groups) All elements whose currents are to be eliminated will be
/// referred to as being in group 1, while all other elements will be referred to as group 2.
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

impl Element {
    /// Retrieves the nodes associated with the element.
    pub fn nodes(&self) -> Vec<String> {
        match self {
            Element::VoltageSource(v) => vec![v.plus.clone(), v.minus.clone()],
            Element::CurrentSource(i) => vec![i.plus.clone(), i.minus.clone()],
            Element::Resistor(r) => vec![r.plus.clone(), r.minus.clone()],
            Element::Capacitor(c) => vec![c.plus.clone(), c.minus.clone()],
            Element::Inductor(l) => vec![l.plus.clone(), l.minus.clone()],
            Element::Diode(d) => vec![d.plus.clone(), d.minus.clone()],
            Element::BJT(b) => vec![b.collector.clone(), b.emitter.clone(), b.base.clone()],
            Element::MOSFET(m) => vec![m.drain.clone(), m.gate.clone(), m.source.clone()],
        }
    }

    pub fn set_value(&mut self, value: f64) {
        match self {
            Element::VoltageSource(e) => e.value = value,
            Element::CurrentSource(e) => e.value = value,
            Element::Resistor(e) => e.value = value,
            Element::Capacitor(e) => e.value = value,
            Element::Inductor(e) => e.value = value,
            Element::Diode(_) => todo!(),
            Element::BJT(_) => todo!(),
            Element::MOSFET(_) => todo!(),
        }
    }

    pub fn is_g2(&self) -> bool {
        match self {
            Element::VoltageSource(_) => true,
            Element::CurrentSource(e) => e.g2,
            Element::Resistor(e) => e.g2,
            Element::Capacitor(e) => e.g2,
            Element::Inductor(_) => true,
            Element::Diode(_) => false,
            Element::BJT(_) => false,
            Element::MOSFET(_) => false,
        }
    }

    pub fn add_dc_stamp(&self, mna_matrix: &mut MnaMatrix) {
        match self {
            Element::VoltageSource(e) => e.add_dc_stamp(mna_matrix),
            Element::CurrentSource(e) => e.add_dc_stamp(mna_matrix),
            Element::Resistor(e) => e.add_dc_stamp(mna_matrix),
            Element::Capacitor(_) => todo!(),
            Element::Inductor(e) => e.add_dc_stamp(mna_matrix),
            Element::Diode(_) => todo!(),
            Element::BJT(_) => todo!(),
            Element::MOSFET(_) => todo!(),
        }
    }

    pub fn add_ac_stamp(&self, mna_matrix: &mut MnaMatrix, frequency: f64) {
        match self {
            Element::VoltageSource(_) => todo!(),
            Element::CurrentSource(_) => todo!(),
            Element::Resistor(_) => todo!(),
            Element::Capacitor(_) => todo!(),
            Element::Inductor(_) => todo!(),
            Element::Diode(_) => todo!(),
            Element::BJT(_) => todo!(),
            Element::MOSFET(_) => todo!(),
        }
    }

    pub fn identifier(&self) -> String {
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

pub trait Identifiable {
    fn identifier(&self) -> String;
}
pub trait Stampable {
    fn add_dc_stamp(&self, mna_matrix: &mut MnaMatrix);
    fn add_ac_stamp(&self, mna_matrix: &mut MnaMatrix, frequency: f64) {
        todo!()
    }
}

impl std::fmt::Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Element::VoltageSource(v) => write!(f, "{}", v.identifier()),
            Element::CurrentSource(i) => write!(f, "{}", i.identifier()),
            Element::Resistor(r) => write!(f, "{}", r.identifier()),
            Element::Capacitor(c) => write!(f, "{}", c.identifier()),
            Element::Inductor(l) => write!(f, "{}", l.identifier()),
            Element::Diode(d) => write!(f, "{}", d.identifier()),
            Element::BJT(b) => write!(f, "{}", b.identifier()),
            Element::MOSFET(m) => write!(f, "{}", m.identifier()),
        }
    }
}
