use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::circuit::Circuit;
use crate::elements::Element;
use crate::prelude::*;

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
/// use crate::krets_parser::parser::parse_circuit_description;
/// let circuit_description = "V1 1 2 1000\n";
/// let netlist = parse_circuit_description(circuit_description).unwrap();
/// ```
///
/// # Errors
/// - Returns an error if a line has an invalid format.
/// - Returns an error if a node name is not a non-negative integer.
/// - Returns an error if a component type is unrecognized.
pub fn parse_circuit_description(input: &str) -> Result<Circuit> {
    let lines: Vec<&str> = input.lines().collect();

    let mut elements: Vec<Element> = Vec::new();
    let mut index_map: HashMap<String, usize> = HashMap::new();
    let mut nodes: Vec<String> = vec![];
    let mut index_counter = 0;
    let mut inside_control_block = false;

    for line in lines {
        if line.is_empty() || line.starts_with('%') || line.starts_with('*') {
            continue;
        }

        if line.starts_with(".control") {
            inside_control_block = true;
            continue;
        }

        if line.starts_with(".endc") {
            inside_control_block = false;
            continue;
        }

        if inside_control_block {
            continue;
        }

        let element = if line.starts_with("V") || line.starts_with("v") {
            Element::VoltageSource(line.parse()?)
        } else if line.starts_with("I") || line.starts_with("i") {
            Element::CurrentSource(line.parse()?)
        } else if line.starts_with("R") || line.starts_with("r") {
            Element::Resistor(line.parse()?)
        } else if line.starts_with("C") || line.starts_with("c") {
            Element::Capacitor(line.parse()?)
        } else if line.starts_with("L") || line.starts_with("l") {
            Element::Inductor(line.parse()?)
        } else if line.starts_with("D") || line.starts_with("d") {
            Element::Diode(line.parse()?)
        } else if line.starts_with("Q") || line.starts_with("q") {
            Element::BJT(line.parse()?)
        } else if line.starts_with("M") || line.starts_with("m") {
            Element::MOSFET(line.parse()?)
        } else {
            continue;
        };

        if element.is_g2() {
            index_map.insert(format!("I({})", element), index_counter);
            index_counter += 1;
        }

        for node in &element.nodes() {
            if !nodes.contains(node) {
                nodes.push(node.clone());
            }

            let index_name = format!("V({})", node);

            // Skip the ground node.
            if node == "0" || index_map.contains_key(&index_name) {
                continue;
            }

            index_map.insert(index_name, index_counter);
            index_counter += 1;
        }

        elements.push(element);
    }

    if elements.is_empty() {
        return Err(Error::EmptyNetlist);
    }
    let netlist = Circuit::new(elements, index_map, nodes);

    Ok(netlist)
}

pub fn parse_circuit_description_file(file_path: &Path) -> Result<Circuit> {
    let file = File::open(file_path).map_err(|e| Error::Unexpected(e.to_string()))?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader
        .read_to_string(&mut contents)
        .map_err(|e| Error::Unexpected(e.to_string()))?;
    parse_circuit_description(&contents)
}
