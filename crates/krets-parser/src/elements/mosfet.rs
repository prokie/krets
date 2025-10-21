use crate::{elements::Stampable, prelude::*};
use nom::{
    IResult,
    Parser,
    branch::alt,
    bytes::complete::tag,
    character::complete::{digit1, space1}, // Added digit1
    combinator::{all_consuming, map_res},
    sequence::preceded,
};
use std::str::FromStr;

use super::Identifiable;

#[derive(Debug, PartialEq, Clone)]
/// Represents the type of a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor).
/// A MOSFET can be either an N-Channel or a P-Channel.
pub enum MosfetType {
    /// N-Channel MOSFET.
    NChannel,
    /// P-Channel MOSFET.
    PChannel,
}

#[derive(Debug, Clone)]
/// Represents a MOSFET (Metal-Oxide-Semiconductor Field-Effect Transistor) in a circuit.
/// SPICE format: M<name> <drain> <gate> <source> <bulk/substrate> <model> [parameters...]
pub struct MOSFET {
    /// Name of the MOSFET (numeric part).
    pub name: u32,
    /// Drain node of the MOSFET.
    pub drain: String,
    /// Gate node of the MOSFET.
    pub gate: String,
    /// Source node of the MOSFET.
    pub source: String,
    /// Bulk (or Substrate) node of the MOSFET.
    pub bulk: String, // Added bulk node
    /// Model name associated with the MOSFET (required).
    pub model_name: String,
    /// Type of the MOSFET (inferred from name, e.g., MN or MP).
    pub mosfet_type: MosfetType,
}

impl Identifiable for MOSFET {
    /// Returns the identifier of the MOSFET in the format `M{name}`.
    fn identifier(&self) -> String {
        format!("M{}", self.name)
    }
}

impl Stampable for MOSFET {
    fn add_conductance_matrix_dc_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
        _solution_map: &std::collections::HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        todo!()
    }

    fn add_excitation_vector_dc_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
        _solution_map: &std::collections::HashMap<String, f64>,
    ) -> Vec<faer::sparse::Triplet<usize, usize, f64>> {
        todo!()
    }

    fn add_excitation_vector_ac_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
        _solution_map: &std::collections::HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, faer::c64>> {
        vec![]
    }

    fn add_conductance_matrix_ac_stamp(
        &self,
        _index_map: &std::collections::HashMap<String, usize>,
        _solution_map: &std::collections::HashMap<String, f64>,
        _frequency: f64,
    ) -> Vec<faer::sparse::Triplet<usize, usize, faer::c64>> {
        todo!()
    }
}

// Nom parser for MOSFET (updated for bulk node)
fn parse_mosfet(input: &str) -> IResult<&str, MOSFET> {
    // Parse the initial 'M' (case-insensitive)
    let (input, _) = alt((tag("M"), tag("m"))).parse(input)?;

    // Parse the type character (N or P, case-insensitive) - kept for structuring data
    let (input, type_char) = alt((tag("N"), tag("n"), tag("P"), tag("p"))).parse(input)?;
    let mosfet_type = match type_char.to_ascii_uppercase().as_str() {
        "N" => MosfetType::NChannel,
        "P" => MosfetType::PChannel,
        _ => unreachable!(),
    };

    // Parse the numeric name part
    let (input, name) = map_res(digit1, |s: &str| s.parse::<u32>()).parse(input)?;

    // Parse nodes: drain, gate, source, bulk
    let (input, drain) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, gate) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, source) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, bulk) = preceded(space1, alphanumeric_or_underscore1).parse(input)?; // Added bulk parser

    // Parse the required model name
    let (input, model_name) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;

    // NOTE: Removed optional value parser, as it's not standard for the main definition line.
    // Parameters like W, L are usually separate or on the model card.

    let mosfet = MOSFET {
        name,
        drain: drain.to_string(),
        gate: gate.to_string(),
        source: source.to_string(),
        bulk: bulk.to_string(), // Added bulk field
        model_name: model_name.to_string(),
        mosfet_type,
    };

    Ok((input, mosfet))
}

impl FromStr for MOSFET {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();
        if s_without_comment.is_empty() {
            return Err(Error::InvalidFormat(
                "Empty line after comment removal".to_string(),
            ));
        }

        // Expected format: M<name> <drain> <gate> <source> <bulk> <model>
        match all_consuming(parse_mosfet).parse(s_without_comment) {
            Ok((_, mosfet)) => {
                if mosfet.name == 0 {
                    return Err(Error::InvalidFormat(format!(
                        "MOSFET name cannot be zero: '{s}'"
                    )));
                }
                Ok(mosfet)
            }
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(Error::InvalidFormat(format!(
                "Failed to parse MOSFET line '{}': {:?}. Expected format: M<name> D G S B <model>", // Updated error message
                s_without_comment, e.code
            ))),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidFormat(format!(
                "Incomplete parse for MOSFET line: '{}'. Expected format: M<name> D G S B <model>", // Updated error message
                s_without_comment
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nchannel_mosfet() {
        // Standard SPICE format: M<name> <drain> <gate> <source> <bulk> <model>
        let mosfet_str = "MN1 D G S B MyNmosModel"; // Added bulk node 'B'
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 1);
        assert_eq!(mosfet.drain, "D");
        assert_eq!(mosfet.gate, "G");
        assert_eq!(mosfet.source, "S");
        assert_eq!(mosfet.bulk, "B"); // Check bulk node
        assert_eq!(mosfet.model_name, "MyNmosModel");
        assert_eq!(mosfet.mosfet_type, MosfetType::NChannel);
    }

    #[test]
    fn test_parse_pchannel_mosfet_case_insensitive() {
        let mosfet_str = "mp5 4 5 6 0 PModel"; // Lowercase 'm' and 'p', bulk tied to ground (0)
        let mosfet = mosfet_str.parse::<MOSFET>().unwrap();

        assert_eq!(mosfet.name, 5);
        assert_eq!(mosfet.drain, "4");
        assert_eq!(mosfet.gate, "5");
        assert_eq!(mosfet.source, "6");
        assert_eq!(mosfet.bulk, "0"); // Check bulk node
        assert_eq!(mosfet.model_name, "PModel");
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "MP3 1 2 3 0 P_FET % My P-Channel FET"; // Added bulk node 0
        let mosfet = s.parse::<MOSFET>().unwrap();
        assert_eq!(mosfet.name, 3);
        assert_eq!(mosfet.mosfet_type, MosfetType::PChannel);
        assert_eq!(mosfet.model_name, "P_FET");
        assert_eq!(mosfet.bulk, "0");
    }

    #[test]
    fn test_invalid_mosfet_format_missing_bulk() {
        let mosfet_str = "MN1 1 2 3 MyModel"; // Missing bulk node
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_missing_model() {
        let mosfet_str = "MN1 1 2 3 0"; // Missing model name
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_format_too_few_nodes() {
        let mosfet_str = "MN1 1 2 MyModel";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_type_char() {
        let s = "MX1 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_name_not_numeric() {
        let s = "MNA 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 2 3 0 MyModel";
        let result = s.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_mosfet_name_zero() {
        let mosfet_str = "MN0 1 2 3 0 MyModel";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_mosfet_with_optional_value_removed() {
        // This format is no longer supported by the parser
        let mosfet_str = "MN2 7 8 9 0 N_Model 1.5";
        let result = mosfet_str.parse::<MOSFET>();
        assert!(result.is_err()); // Should fail because "1.5" is an extra part
    }
}
