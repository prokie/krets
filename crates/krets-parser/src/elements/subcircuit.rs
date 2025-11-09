use crate::prelude::*;
use nom::{
    IResult, Parser, bytes::complete::tag_no_case, character::complete::space1, multi::many0,
    sequence::preceded,
};
#[derive(Debug, Clone)]
pub struct SubcircuitDefinition {
    pub name: String,
    pub pins: Vec<String>,
    pub elements: Vec<Element>,
}

#[derive(Debug, Clone)]
pub struct SubcircuitInstance {
    pub instance_name: String,
    pub definition_name: String,
    pub nodes: Vec<String>,
}

impl SubcircuitInstance {
    pub fn new(
        instance_name: impl Into<String>,
        definition_name: impl Into<String>,
        nodes: Vec<&str>,
    ) -> Self {
        Self {
            instance_name: instance_name.into(),
            definition_name: definition_name.into(),
            nodes: nodes.into_iter().map(Into::into).collect(),
        }
    }

    pub fn instantiate(
        &self,
        definitions: &HashMap<String, SubcircuitDefinition>,
    ) -> Result<Vec<Element>> {
        let mut final_elements: Vec<Element> = Vec::new();

        // 1. Find the definition for this instance
        let definition = definitions.get(&self.definition_name).ok_or_else(|| {
            Error::InvalidFormat(format!(
                "Undefined subcircuit definition: {}",
                self.definition_name
            ))
        })?;

        // 2. Create the node mapping for this level
        if self.nodes.len() != definition.pins.len() {
            return Err(Error::InvalidFormat(format!(
                "Node/port mismatch for instance {}: expected {}, got {}",
                self.instance_name,
                definition.pins.len(),
                self.nodes.len()
            )));
        }
        let port_to_node: HashMap<&String, &String> =
            definition.pins.iter().zip(self.nodes.iter()).collect();

        // 3. Iterate over all elements inside the definition
        for sub_element in &definition.elements {
            // 4. Instantiate the nodes and name of this sub-element
            let mapped_element = map_sub_element(sub_element, &port_to_node, &self.instance_name)?;

            // 5. Check if the mapped element is *another* subcircuit or a primitive
            match mapped_element {
                Element::SubcktInstance(next_instance) => {
                    // It's another subcircuit, recurse by calling the method on the nested instance
                    let mut expanded_elements = next_instance.instantiate(definitions)?;
                    final_elements.append(&mut expanded_elements);
                }
                _ => {
                    // It's a primitive, add it to our list
                    final_elements.push(mapped_element);
                } // Add other primitive types here in the future
            }
        }

        Ok(final_elements)
    }
}

/// This function maps nodes and prefixes the name for a *single* element
/// from a subcircuit definition.
pub fn map_sub_element(
    subckt_element: &Element,
    port_to_node: &HashMap<&String, &String>,
    parent_instance_name: &str,
) -> Result<Element> {
    // Clone the subcircuit element to modify
    let mut instantiated_element = subckt_element.clone();

    // Update the nodes of the instantiated element
    for node in instantiated_element.nodes_mut() {
        if let Some(actual_node) = port_to_node.get(node) {
            *node = (*actual_node).clone();
        }
    }

    // Prefix the instance name to the element name for uniqueness
    instantiated_element.set_name(&format!(
        "{}_{}",
        parent_instance_name,
        instantiated_element.name()
    ));

    Ok(instantiated_element)
}

impl Identifiable for SubcircuitInstance {
    fn identifier(&self) -> String {
        format!("X{}", self.instance_name)
    }
}

impl Stampable for SubcircuitInstance {
    fn stamp_conductance_matrix_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        todo!()
    }

    fn stamp_excitation_vector_dc(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
    ) -> Vec<Triplet<usize, usize, f64>> {
        todo!()
    }

    fn stamp_conductance_matrix_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        todo!()
    }

    fn stamp_excitation_vector_ac(
        &self,
        _index_map: &HashMap<String, usize>,
        _solution_map: &HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<Triplet<usize, usize, c64>> {
        todo!()
    }
}

impl SubcircuitDefinition {
    pub fn new(name: impl Into<String>, pins: Vec<&str>) -> Self {
        Self {
            name: name.into(),
            pins: pins.into_iter().map(Into::into).collect(),
            elements: Vec::new(),
        }
    }
}

pub fn parse_subckt_header(input: &str) -> IResult<&str, SubcircuitDefinition> {
    let (input, _) = tag_no_case(".subckt").parse(input)?;
    let (input, name) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, pins) = many0(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;
    Ok((input, SubcircuitDefinition::new(name, pins)))
}

pub fn parse_subckt_instance(input: &str) -> IResult<&str, SubcircuitInstance> {
    let (input, _) = tag_no_case("x").parse(input)?;
    let (input, instance_name) = alphanumeric_or_underscore1(input)?;
    let (input, nodes) = many0(preceded(space1, alphanumeric_or_underscore1)).parse(input)?;

    if nodes.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::LengthValue,
        )));
    }

    let definition_name = nodes.last().unwrap();
    let nodes = &nodes[..nodes.len() - 1];
    Ok((
        input,
        SubcircuitInstance::new(
            instance_name.to_string(),
            definition_name.to_string(),
            nodes.to_vec(),
        ),
    ))
}

pub fn parse_subcircuits(input: &str) -> Result<HashMap<String, SubcircuitDefinition>> {
    let mut subcircuit_definitions: HashMap<String, SubcircuitDefinition> = HashMap::new();
    let mut inside_subckt_block = false;
    let mut current_subckt_name = String::new();

    for line in input.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('%') || line.starts_with('*') {
            continue;
        }

        if line.to_lowercase().starts_with(".subckt") {
            let (_, subckt_header) = parse_subckt_header(line).map_err(|e| {
                Error::InvalidFormat(format!("Failed to parse subcircuit header: {}", e))
            })?;

            current_subckt_name = subckt_header.name.clone();
            subcircuit_definitions.insert(subckt_header.name.clone(), subckt_header);

            inside_subckt_block = true;

            continue;
        }

        if line.to_lowercase().starts_with(".ends") {
            inside_subckt_block = false;
            current_subckt_name.clear();
            continue;
        }

        if inside_subckt_block {
            // We now use parse_element, which can handle primitives (r) AND
            // nested subcircuit instances (x)
            let subckt_element = parse_element(line).map_err(|e| {
                Error::InvalidFormat(format!(
                    "Failed to parse subcircuit element in '{}': {}",
                    current_subckt_name, e
                ))
            })?;

            if let Some(subckt_def) = subcircuit_definitions.get_mut(&current_subckt_name) {
                subckt_def.elements.push(subckt_element);
            }
            continue;
        }
    }

    Ok(subcircuit_definitions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header() {
        let subckt_str = ".SUBCKT my_subckt in out vdd gnd";
        let (_, subckt) = parse_subckt_header(subckt_str).unwrap();
        assert_eq!(subckt.name, "my_subckt");
        assert_eq!(subckt.pins, vec!["in", "out", "vdd", "gnd"]);
    }
}
