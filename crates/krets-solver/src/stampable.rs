use crate::prelude::*;
use krets_parser::elements::{
    Element, bjt::BJT, capacitor::Capacitor, current_source::CurrentSource, diode::Diode,
    inductor::Inductor, nmosfet::NMOSFET, resistor::Resistor, subcircuit::SubcircuitInstance,
    voltage_source::VoltageSource,
};

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

impl Stampable for Resistor {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets;

        if self.g2 {
            triplets = Vec::with_capacity(5);
            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                triplets.push(Triplet::new(index_plus, index_current, 1.0));
                triplets.push(Triplet::new(index_current, index_plus, 1.0));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                triplets.push(Triplet::new(index_minus, index_current, -1.0));
                triplets.push(Triplet::new(index_current, index_minus, -1.0));
            }

            if let Some(&index_current) = index_current {
                triplets.push(Triplet::new(index_current, index_current, -self.value));
            }
        } else {
            triplets = Vec::with_capacity(4);

            let g = 1.0 / self.value;
            if let Some(&ip) = index_plus {
                triplets.push(Triplet::new(ip, ip, g));
            }
            if let Some(&im) = index_minus {
                triplets.push(Triplet::new(im, im, g));
            }

            if let (Some(&ip), Some(&im)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(ip, im, -g));
                triplets.push(Triplet::new(im, ip, -g));
            }
        }
        triplets
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets;

        if self.g2 {
            triplets = Vec::with_capacity(5);
            let one = c64::new(1.0, 0.0);
            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                triplets.push(Triplet::new(index_plus, index_current, one));
                triplets.push(Triplet::new(index_current, index_plus, one));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                triplets.push(Triplet::new(index_minus, index_current, -one));
                triplets.push(Triplet::new(index_current, index_minus, -one));
            }

            if let Some(&index_current) = index_current {
                triplets.push(Triplet::new(
                    index_current,
                    index_current,
                    -c64::new(self.value, 0.0),
                ));
            }
        } else {
            triplets = Vec::with_capacity(4);
            let g = c64::new(1.0 / self.value, 0.0);
            if let Some(&ip) = index_plus {
                triplets.push(Triplet::new(ip, ip, g));
            }
            if let Some(&im) = index_minus {
                triplets.push(Triplet::new(im, im, g));
            }

            if let (Some(&ip), Some(&im)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(ip, im, -g));
                triplets.push(Triplet::new(im, ip, -g));
            }
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // A resistor is a passive component and does not add to the excitation vector.
        Vec::new()
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        // A resistor is a passive component and does not add to the excitation vector.
        Vec::new()
    }
}

impl Stampable for BJT {
    // --- Stamping methods remain unchanged ---
    fn stamp_conductance_matrix_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // TODO: Implement BJT DC conductance stamp
        todo!()
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // TODO: Implement BJT DC excitation stamp
        todo!()
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // BJTs are passive for small-signal AC excitation
        vec![]
    }

    fn stamp_conductance_matrix_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // TODO: Implement BJT AC conductance stamp (small-signal model)
        todo!()
    }
}

impl Stampable for Capacitor {
    fn stamp_conductance_matrix_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // A capacitor is an open circuit in DC analysis, so it contributes nothing to the DC conductance matrix.
        vec![]
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let admittance = c64 {
            re: 0.0,
            im: 2.0 * PI * frequency * self.value,
        };

        let mut triplets = Vec::with_capacity(4);

        if !self.g2 {
            if let Some(&index_plus) = index_plus {
                triplets.push(Triplet::new(index_plus, index_plus, admittance));
            }
            if let Some(&index_minus) = index_minus {
                triplets.push(Triplet::new(index_minus, index_minus, admittance));
            }
            if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
                triplets.push(Triplet::new(index_plus, index_minus, -admittance));
                triplets.push(Triplet::new(index_minus, index_plus, -admittance));
            }
        } else {
            let index_current = index_map.get(&format!("I({})", self.identifier()));

            if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
                // -Y contribution for V_plus
                triplets.push(Triplet::new(index_current, index_plus, -admittance));
            }

            if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
                // +Y contribution for V_minus
                triplets.push(Triplet::new(index_current, index_minus, admittance));
            }

            if let Some(&index_current) = index_current {
                // +1 contribution for I_c
                triplets.push(Triplet::new(
                    index_current,
                    index_current,
                    c64 { re: 1.0, im: 0.0 },
                ));
            }
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        // Capacitors are passive and don't contribute to the DC excitation vector.
        vec![]
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, c64>> {
        // Capacitors are passive and don't contribute to the AC excitation vector.
        vec![]
    }

    fn stamp_conductance_matrix_transient(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>, // Not needed for a linear capacitor's conductance
        _prev_solution: &HashMap<String, f64>, // Not needed for a linear capacitor's conductance
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let g = self.value / h;

        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let mut triplets = Vec::with_capacity(4);

        if let Some(&ip) = index_plus {
            triplets.push(Triplet::new(ip, ip, g));
        }
        if let Some(&im) = index_minus {
            triplets.push(Triplet::new(im, im, g));
        }
        if let (Some(&ip), Some(&im)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(ip, im, -g));
            triplets.push(Triplet::new(im, ip, -g));
        }

        triplets
    }

    fn stamp_excitation_vector_transient(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>, // Not needed for a linear capacitor's excitation
        prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // Get the capacitor's voltage from the PREVIOUS time step.
        // Default to 0.0 if a node is not in the map (e.g., ground or first step).
        let v_plus_prev = prev_solution
            .get(&format!("V({})", self.plus))
            .copied()
            .unwrap_or(0.0);
        let v_minus_prev = prev_solution
            .get(&format!("V({})", self.minus))
            .copied()
            .unwrap_or(0.0);
        let v_prev = v_plus_prev - v_minus_prev;

        // Calculate the equivalent current source value: I_eq = (C/h) * v_prev
        let i_eq = -(self.value / h) * v_prev;

        let mut triplets = Vec::with_capacity(2);

        if let Some(&ip) = index_plus {
            triplets.push(Triplet::new(ip, 0, -i_eq));
        }
        if let Some(&im) = index_minus {
            triplets.push(Triplet::new(im, 0, i_eq));
        }

        triplets
    }
}

impl Stampable for CurrentSource {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(3);
        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(index_current, index_current, 1.0));
        }
        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
        }
        triplets
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented the AC stamp, which is identical to the DC stamp for a
        // frequency-independent current source.
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(3);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, c64::new(1.0, 0.0)));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_minus,
                index_current,
                c64::new(-1.0, 0.0),
            ));
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        match index_map.get(&format!("I({})", self.identifier())) {
            Some(i) => vec![Triplet::new(*i, 0, self.value)],
            None => Vec::new(),
        }
    }

    fn stamp_excitation_vector_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented the AC excitation stamp. For a simple source, this is a
        // real value, but it's represented as a complex number.
        match index_map.get(&format!("I({})", self.identifier())) {
            Some(i) => vec![Triplet::new(*i, 0, c64::new(self.value, 0.0))],
            None => Vec::new(),
        }
    }
}

impl Stampable for Diode {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        // The linearized conductance of the diode for the current iteration.
        let conductance = self.conductance(solution_map);

        let mut triplets = Vec::with_capacity(4);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance));
        }
        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance));
        }
        triplets
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // FIX: Implemented AC stamp. For a diode, the small-signal AC conductance
        // at a given DC bias point is the same as its linearized DC conductance.
        let conductance = self.conductance(solution_map);
        let conductance_complex = c64::new(conductance, 0.0);

        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let mut triplets = Vec::with_capacity(4);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, index_plus, conductance_complex));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, index_minus, conductance_complex));
        }
        if let (Some(&index_plus), Some(&index_minus)) = (index_plus, index_minus) {
            triplets.push(Triplet::new(index_plus, index_minus, -conductance_complex));
            triplets.push(Triplet::new(index_minus, index_plus, -conductance_complex));
        }
        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));

        let equivalent_current = self.equivalent_current(solution_map);

        let mut triplets = Vec::with_capacity(2);

        if let Some(&index_plus) = index_plus {
            triplets.push(Triplet::new(index_plus, 0, -equivalent_current));
        }
        if let Some(&index_minus) = index_minus {
            triplets.push(Triplet::new(index_minus, 0, equivalent_current));
        }
        triplets
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        // A diode is a passive component and does not
        // contribute to the excitation vector in small-signal AC analysis.
        vec![]
    }
}

impl Stampable for Inductor {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(4);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, 1.0));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_minus, index_current, -1.0));
        }

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_current, index_plus, 1.0));
        }
        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(index_current, index_minus, -1.0));
        }

        triplets
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));
        let impedance = c64::new(0.0, 2.0 * PI * frequency * self.value);
        let mut triplets = Vec::with_capacity(5);

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_plus, index_current, c64::new(1.0, 0.0)));
        }

        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_minus,
                index_current,
                c64::new(-1.0, 0.0),
            ));
        }

        if let (Some(&index_plus), Some(&index_current)) = (index_plus, index_current) {
            triplets.push(Triplet::new(index_current, index_plus, c64::new(1.0, 0.0)));
        }
        if let (Some(&index_minus), Some(&index_current)) = (index_minus, index_current) {
            triplets.push(Triplet::new(
                index_current,
                index_minus,
                c64::new(-1.0, 0.0),
            ));
        }

        if let Some(&index_current) = index_current {
            triplets.push(Triplet::new(index_current, index_current, -impedance));
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        // An ideal inductor is passive and has no internal sources.
        vec![]
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        // An ideal inductor is passive and has no internal sources.
        vec![]
    }

    fn stamp_conductance_matrix_transient(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(5);

        if let Some(&ic) = index_current {
            triplets.push(Triplet::new(ic, ic, -self.value / h));
        }

        if let (Some(&ip), Some(&ic)) = (index_plus, index_current) {
            triplets.push(Triplet::new(ip, ic, 1.0));
            triplets.push(Triplet::new(ic, ip, 1.0));
        }

        if let (Some(&im), Some(&ic)) = (index_minus, index_current) {
            triplets.push(Triplet::new(im, ic, -1.0));
            triplets.push(Triplet::new(ic, im, -1.0));
        }

        triplets
    }

    fn stamp_excitation_vector_transient(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        prev_solution: &HashMap<String, f64>,
        h: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let i_prev = prev_solution
            .get(&format!("I({})", self.identifier()))
            .copied()
            .unwrap();

        if let Some(&ic) = index_current {
            vec![Triplet::new(ic, 0, -(self.value / h) * i_prev)]
        } else {
            vec![]
        }
    }
}

impl Stampable for NMOSFET {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let v_g = solution_map
            .get(&format!("V({})", self.gate))
            .unwrap_or(&0.0);
        let v_s = solution_map
            .get(&format!("V({})", self.source))
            .unwrap_or(&0.0);
        let v_d = solution_map
            .get(&format!("V({})", self.drain))
            .unwrap_or(&0.0);

        let v_gs = v_g - v_s;
        let v_ds = v_d - v_s;

        let mut triplets = Vec::new();
        let g_m = self.g_m(v_gs, v_ds);
        let g_ds = self.g_ds(v_gs, v_ds);

        let index_d = index_map.get(&self.drain);
        let index_s = index_map.get(&self.source);
        let index_g = index_map.get(&self.gate);

        if let Some(&id) = index_d {
            triplets.push(Triplet::new(id, id, g_ds));
        }

        if let Some(&is) = index_s {
            triplets.push(Triplet::new(is, is, g_ds + g_m));
        }

        if let (Some(&id), Some(&is)) = (index_d, index_s) {
            triplets.push(Triplet::new(id, is, -(g_ds + g_m)));
            triplets.push(Triplet::new(is, id, g_ds + g_m));
        }

        if let (Some(&is), Some(&ig)) = (index_s, index_g) {
            triplets.push(Triplet::new(is, ig, g_m));
        }

        if let (Some(&id), Some(&ig)) = (index_d, index_g) {
            triplets.push(Triplet::new(id, ig, g_m));
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let v_g = solution_map
            .get(&format!("V({})", self.gate))
            .unwrap_or(&0.0);
        let v_s = solution_map
            .get(&format!("V({})", self.source))
            .unwrap_or(&0.0);
        let v_d = solution_map
            .get(&format!("V({})", self.drain))
            .unwrap_or(&0.0);

        let v_gs = v_g - v_s;
        let v_ds = v_d - v_s;
        let g_ds = self.g_ds(v_gs, v_ds);
        let g_m = self.g_m(v_gs, v_ds);
        let i_d = self.i_d(v_gs, v_ds);

        let i_eq = i_d - g_ds * v_ds - g_m * v_gs;

        let mut triplets = Vec::new();

        if let Some(&is) = index_map.get(&self.source) {
            triplets.push(Triplet::new(is, 0, i_eq));
        }

        if let Some(&id) = index_map.get(&self.drain) {
            triplets.push(Triplet::new(id, 0, -i_eq));
        }
        triplets
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        vec![]
    }

    fn stamp_conductance_matrix_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, faer::c64>> {
        todo!()
    }
}

impl Stampable for SubcircuitInstance {
    fn stamp_conductance_matrix_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        unreachable!("Subcircuit instances should be expanded before stamping")
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        unreachable!("Subcircuit instances should be expanded before stamping")
    }

    fn stamp_conductance_matrix_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        unreachable!("Subcircuit instances should be expanded before stamping")
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        unreachable!("Subcircuit instances should be expanded before stamping")
    }
}

impl Stampable for VoltageSource {
    fn stamp_conductance_matrix_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));

        let mut triplets = Vec::with_capacity(4);

        if let (Some(&ip), Some(&ic)) = (index_plus, index_current) {
            triplets.push(Triplet::new(ip, ic, 1.0));
            triplets.push(Triplet::new(ic, ip, 1.0));
        }

        if let (Some(&im), Some(&ic)) = (index_minus, index_current) {
            triplets.push(Triplet::new(im, ic, -1.0));
            triplets.push(Triplet::new(ic, im, -1.0));
        }

        triplets
    }

    fn stamp_conductance_matrix_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let index_plus = index_map.get(&format!("V({})", self.plus));
        let index_minus = index_map.get(&format!("V({})", self.minus));
        let index_current = index_map.get(&format!("I({})", self.identifier()));
        let one = c64::new(1.0, 0.0);
        let mut triplets = Vec::with_capacity(4);

        if let (Some(&ip), Some(&ic)) = (index_plus, index_current) {
            triplets.push(Triplet::new(ip, ic, one));
            triplets.push(Triplet::new(ic, ip, one));
        }

        if let (Some(&im), Some(&ic)) = (index_minus, index_current) {
            triplets.push(Triplet::new(im, ic, -one));
            triplets.push(Triplet::new(ic, im, -one));
        }

        triplets
    }

    fn stamp_excitation_vector_dc(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let mut triplets = Vec::with_capacity(1);
        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            triplets.push(Triplet::new(ic, 0, self.dc_value));
        }
        triplets
    }

    fn stamp_excitation_vector_ac(
        &self,
        index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        let mut triplets = Vec::with_capacity(1);

        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            triplets.push(Triplet::new(ic, 0, c64::new(self.ac_amplitude, 0.0)));
        }
        triplets
    }

    fn stamp_excitation_vector_transient(
        &self,
        index_map: &HashMap<String, usize>,
        solution_map: &HashMap<String, f64>,
        _prev_solution: &HashMap<String, f64>,
        _time_step: f64,
    ) -> Vec<Triplet<usize, usize, f64>> {
        let current_time = solution_map.get("time").cloned().unwrap_or(0.0);
        if let Some(&ic) = index_map.get(&format!("I({})", self.identifier())) {
            vec![Triplet::new(ic, 0, self.transient_value_at(current_time))]
        } else {
            vec![]
        }
    }
}
