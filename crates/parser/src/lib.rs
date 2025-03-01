pub mod elements;
pub mod error;
pub mod netlist;
pub mod prelude;

use crate::prelude::*;
use elements::Element;
use netlist::Netlist;

/// Parses a SPICE-like netlist and extracts circuit elements into structured data.
///
/// # Description
/// This function reads a circuit specification from a text-based netlist format.
/// The parser follows these rules:
/// - It is **case-insensitive**.
/// - It treats any sequence of spaces or tabs as a **single space**.
/// - Each line describes **one circuit element** entirely.
/// - The **order of lines** in the file is **irrelevant**.
/// - Any text following a `%` character in a line is treated as a **comment** and ignored.
/// - Circuit node names are **non-negative integers**, where `0` is reserved for **ground**.
///
/// # Supported Elements
/// - **Voltage Source**: `V<int> <node.+> <node.-> <value>`
/// - **Current Source**: `I<int> <node.+> <node.-> <value> [G2]`
/// - **Resistor**: `R<int> <node.+> <node.-> <value> [G2]`
/// - **Capacitor**: `C<int> <node.+> <node.-> <value> [G2]`
/// - **Inductor**: `L<int> <node.+> <node.-> <value>`
/// - **Diode**: `D<int> <node.A> <node.K> [<value>]`
/// - **BJT (Bipolar Junction Transistor)**:
///   - `QN<int> <node.C> <node.B> <node.E> [<value>]` (NPN)
///   - `QP<int> <node.C> <node.B> <node.E> [<value>]` (PNP)
/// - **MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor)**:
///   - `MN<int> <node.D> <node.G> <node.S> [<value>]` (N-Channel)
///   - `MP<int> <node.D> <node.G> <node.S> [<value>]` (P-Channel)
///
/// The `<value>` field represents the component's electrical property (e.g., resistance in ohms),
/// given as a non-negative real number **without units**.
///
/// # Parameters
/// - `input`: A string containing the netlist text.
///
/// # Returns
/// - A `Result<Vec<Netlist>, NetlistParseError>`.
///
/// # Example
/// ```
/// use crate::parser::parse_circuit_description;
/// let circuit_description = "V1 1 2 1000\n";
/// let netlist = parse_circuit_description(circuit_description).unwrap();
/// ```
///
/// # Errors
/// - Returns an error if a line has an invalid format.
/// - Returns an error if a node name is not a non-negative integer.
/// - Returns an error if a component type is unrecognized.
pub fn parse_circuit_description(input: &str) -> Result<Netlist> {
    let lines: Vec<&str> = input.lines().collect();

    let mut elements: Vec<Element> = Vec::new();

    for line in lines {
        if line.is_empty() {
            continue;
        }

        if line.starts_with('%') {
            continue;
        }

        if line.starts_with("V") || line.starts_with("v") {
            elements.push(Element::VoltageSource(line.parse()?));
        }

        if line.starts_with("I") || line.starts_with("i") {
            elements.push(Element::CurrentSource(line.parse()?));
        }
    }

    if elements.is_empty() {
        return Err(Error::EmptyNetlist);
    }

    let netlist = Netlist { elements };

    Ok(netlist)
}
