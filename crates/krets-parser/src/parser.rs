use std::{
    collections::{HashMap, HashSet},
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
/// - Any text following a `%` or `*` character is a **comment** and ignored.
/// - Circuit node names are **non-negative integers**, where `0` is reserved for **ground**.
///
/// # Parameters
/// - `input`: A string containing the netlist text.
///
/// # Returns
/// - A `Result<Circuit, Error>`.
pub fn parse_circuit_description(input: &str) -> Result<Circuit> {
    let mut elements: Vec<Element> = Vec::new();
    let mut index_map: HashMap<String, usize> = HashMap::new();
    let mut nodes: HashSet<String> = HashSet::new();
    let mut index_counter = 0;
    let mut inside_control_block = false;

    for (line_num, line) in input.lines().enumerate() {
        let current_line = line_num + 1;

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

        let parse_with_context = |line: &str| -> Result<Element> {
            if line.starts_with("V") || line.starts_with("v") {
                Ok(Element::VoltageSource(line.parse()?))
            } else if line.starts_with("I") || line.starts_with("i") {
                Ok(Element::CurrentSource(line.parse()?))
            } else if line.starts_with("R") || line.starts_with("r") {
                Ok(Element::Resistor(line.parse()?))
            } else if line.starts_with("C") || line.starts_with("c") {
                Ok(Element::Capacitor(line.parse()?))
            } else if line.starts_with("L") || line.starts_with("l") {
                Ok(Element::Inductor(line.parse()?))
            } else if line.starts_with("D") || line.starts_with("d") {
                Ok(Element::Diode(line.parse()?))
            } else if line.starts_with("Q") || line.starts_with("q") {
                Ok(Element::BJT(line.parse()?))
            } else if line.starts_with("M") || line.starts_with("m") {
                Ok(Element::MOSFET(line.parse()?))
            } else {
                // Continue quietly for lines that are not element definitions
                // This could also be an error if strict parsing is desired.
                Err(Error::Unexpected("Not an element".into()))
            }
        };

        match parse_with_context(line) {
            Ok(element) => {
                if element.is_g2() {
                    index_map.insert(format!("I({element})"), index_counter);
                    index_counter += 1;
                }

                for node in &element.nodes() {
                    if nodes.insert(node.to_string()) {
                        // Skip adding the ground node to the index map
                        if *node == "0" {
                            continue;
                        }

                        let index_name = format!("V({node})");
                        index_map.insert(index_name, index_counter);
                        index_counter += 1;
                    }
                }
                elements.push(element);
            }
            Err(Error::Unexpected(_)) => continue, // Ignore lines that aren't elements
            Err(e) => {
                return Err(Error::ParseError {
                    line: current_line,
                    message: e.to_string(),
                });
            }
        };
    }

    if elements.is_empty() {
        return Err(Error::EmptyNetlist);
    }

    // Convert HashSet to Vec for the final Circuit struct if needed
    let nodes_vec = nodes.into_iter().collect();
    let circuit = Circuit::new(elements, index_map, nodes_vec);

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
