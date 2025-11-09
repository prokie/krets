use crate::{circuit::Circuit, models::Model};
use crate::{elements::Element, models::parse_model};
use crate::{elements::subcircuit::parse_subcircuits, prelude::*};
use std::{
    collections::HashSet,
    fs::File,
    io::{BufReader, Read},
    path::Path,
};
/// Parses a SPICE-like netlist and extracts circuit elements into structured data.
///
/// # Description
/// This function reads a circuit specification from a text-based netlist format.
/// The parser follows these rules:
/// - It is **case-insensitive**.
/// - It treats any sequence of spaces or tabs as a **single space**.
/// - Each line describes **one circuit element** entirely.
/// - The **order of lines** in the file is **irrelevant**.
/// - Any text following a `%` or `*` character is a **comment** and ignored.
/// - Circuit node names are **non-negative integers**, where `0` is reserved for **ground**.
///
/// # Parameters
/// - `input`: A string containing the netlist text.
///
/// # Returns
/// - A `Result<Circuit, Error>`.
pub fn parse_circuit_description(input: &str) -> Result<Circuit> {
    let mut nodes: HashSet<String> = HashSet::new();
    let mut index_counter = 0;
    let mut inside_control_block = false;
    let mut inside_subckt_block = false;
    let mut circuit = Circuit::empty_circuit();

    // First pass: Parse subcircuit definitions
    let subcircuit_definitions = parse_subcircuits(input)
        .map_err(|e| Error::InvalidFormat(format!("Failed to parse subcircuits: {}", e)))?;

    for (line_num, line) in input.lines().enumerate() {
        let current_line = line_num + 1;

        let line = line.trim();

        if line.is_empty() || line.starts_with('%') || line.starts_with('*') {
            continue;
        }

        if line.to_lowercase().starts_with(".control") {
            inside_control_block = true;
            continue;
        }

        if line.to_lowercase().starts_with(".endc") {
            inside_control_block = false;
            continue;
        }
        if inside_control_block {
            continue;
        }

        if line.to_lowercase().starts_with(".subckt") {
            inside_subckt_block = true;
            continue;
        }

        if line.to_lowercase().starts_with(".ends") {
            inside_subckt_block = false;
            continue;
        }

        if inside_subckt_block {
            continue;
        }

        if line.to_lowercase() == ".end" {
            continue;
        }

        if line.to_lowercase().starts_with(".model") {
            let model = parse_model(line).map_err(|e| Error::ParseError {
                line: current_line,
                message: e.to_string(),
            })?;

            circuit.models.insert(model.name().to_string(), model);
            continue;
        }

        let element = parse_element(line).map_err(|e| Error::ParseError {
            line: current_line,
            message: e.to_string(),
        })?;

        match element {
            Element::SubcktInstance(instance) => {
                circuit
                    .elements
                    .append(&mut instance.instantiate(&subcircuit_definitions)?);
            }
            _ => {
                circuit.elements.push(element);
            }
        }
    }

    for element in circuit.elements.iter() {
        if element.is_g2() {
            circuit
                .index_map
                .insert(format!("I({element})"), index_counter);
            index_counter += 1;
        }

        for node in &element.nodes() {
            if nodes.insert(node.to_string()) {
                // Skip adding the ground node to the index map
                if *node == "0" {
                    continue;
                }
                circuit
                    .index_map
                    .insert(format!("V({node})"), index_counter);
                index_counter += 1;
            }
        }
    }

    if circuit.is_empty() {
        return Err(Error::EmptyNetlist);
    }

    // --- Second pass: Apply model parameters to elements ---
    for element in circuit.elements.iter_mut() {
        if let Element::Diode(diode) = element {
            match circuit.models.get(&diode.model_name) {
                Some(Model::Diode(model)) => {
                    diode.model = model.clone();
                }
                _ => todo!(),
            }
        }
        if let Element::NMOSFET(mosfet) = element {
            match circuit.models.get(&mosfet.model_name) {
                Some(Model::NMosfet(model)) => {
                    mosfet.model = model.clone();
                }
                _ => todo!(),
            }
        }
    }

    // Convert HashSet to Vec for the final Circuit struct if needed
    circuit.nodes = nodes.into_iter().collect();
    Ok(circuit)
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
