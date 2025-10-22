// krets-parser/src/model.rs
use crate::prelude::*;
use nom::{
    IResult, Parser,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, multispace0, space0, space1},
    combinator::{all_consuming, map, map_res, opt},
    error::context,
    multi::many0,
    sequence::{delimited, pair, preceded, separated_pair},
};

#[derive(Debug, PartialEq, Clone)]
/// Enum representing the different types of devices supported by the .model card.
pub enum ModelType {
    Resistor,        // R
    Capacitor,       // C
    Inductor,        // L
    VoltageSwitch,   // SW
    CurrentSwitch,   // CSW
    UniformRC,       // URC
    LossyTransLine,  // LTRA
    Diode,           // D
    NpnBjt,          // NPN
    PnpBjt,          // PNP
    NChannelJfet,    // NJF
    PChannelJfet,    // PJF
    NChannelMosfet,  // NMOS
    PChannelMosfet,  // PMOS
    NChannelMesfet,  // NMF
    PChannelMesfet,  // PMF
    PowerMosfet,     // VDMOS
    Unknown(String), // Catch-all for unsupported or future types
}

impl FromStr for ModelType {
    type Err = Error; // Use your crate's error type

    fn from_str(s: &str) -> Result<Self> {
        Ok(match s.to_uppercase().as_str() {
            "R" => ModelType::Resistor,
            "C" => ModelType::Capacitor,
            "L" => ModelType::Inductor,
            "SW" => ModelType::VoltageSwitch,
            "CSW" => ModelType::CurrentSwitch,
            "URC" => ModelType::UniformRC,
            "LTRA" => ModelType::LossyTransLine,
            "D" => ModelType::Diode,
            "NPN" => ModelType::NpnBjt,
            "PNP" => ModelType::PnpBjt,
            "NJF" => ModelType::NChannelJfet,
            "PJF" => ModelType::PChannelJfet,
            "NMOS" => ModelType::NChannelMosfet,
            "PMOS" => ModelType::PChannelMosfet,
            "NMF" => ModelType::NChannelMesfet,
            "PMF" => ModelType::PChannelMesfet,
            "VDMOS" => ModelType::PowerMosfet,
            _ => ModelType::Unknown(s.to_string()), // Or return an error: Err(Error::InvalidFormat(format!("Unknown model type: {}", s))),
        })
    }
}

#[derive(Debug, Clone)]
/// Represents a .model definition from a SPICE netlist.
pub struct Model {
    /// The name assigned to this model.
    pub name: String,
    /// The type of the device this model describes.
    pub model_type: ModelType,
    /// Key-value pairs of parameters defined for this model.
    pub parameters: HashMap<String, f64>,
}

// --- Nom Parsers ---

/// Parses a key=value pair within the model parameters.
fn parse_key_value(input: &str) -> IResult<&str, (&str, f64)> {
    separated_pair(
        alphanumeric_or_underscore1,    // Key (parameter name)
        preceded(space0, tag("=")),     // Separator '=' with optional spaces
        preceded(space0, value_parser), // Value (parsed number)
    )
    .parse(input)
}

/// Parses the parameters within the parentheses `(...)`.
fn parse_parameters(input: &str) -> IResult<&str, HashMap<String, f64>> {
    context(
        "model parameters",
        delimited(
            preceded(space0, tag("(")), // Opening parenthesis with optional leading space
            map(
                many0(preceded(space0, parse_key_value)), // Zero or more key=value pairs, separated by spaces
                |vec| vec.into_iter().map(|(k, v)| (k.to_string(), v)).collect(), // Convert Vec to HashMap
            ),
            preceded(space0, tag(")")), // Closing parenthesis with optional leading space
        ),
    )
    .parse(input)
}

/// Parses the entire .model line.
fn parse_model_line(input: &str) -> IResult<&str, Model> {
    context(
        ".model line",
        preceded(
            // Handle optional starting dot and ensure ".model" is matched case-insensitively
            pair(opt(tag(".")), tag_no_case("model")),
            preceded(
                space1, // Require at least one space after .model
                map_res(
                    (
                        alphanumeric_or_underscore1,             // Model name
                        preceded(space1, alpha1),                // Model type (only letters)
                        preceded(multispace0, parse_parameters), // Optional parameters
                    ),
                    |(mname, mtype_str, params)| {
                        // Attempt to convert the type string into ModelType enum
                        mtype_str.parse::<ModelType>().map(|mtype| Model {
                            name: mname.to_string(),
                            model_type: mtype,
                            parameters: params,
                        })
                        // Map the ModelType parse error to a nom error if needed,
                        // although map_res handles Err conversion implicitly here.
                    },
                ),
            ),
        ),
    )
    .parse(input)
}

impl FromStr for Model {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();
        if s_without_comment.is_empty() {
            return Err(Error::InvalidFormat(
                "Empty line after comment removal".to_string(),
            ));
        }

        match all_consuming(parse_model_line).parse(s_without_comment) {
            Ok((_, model)) => Ok(model),
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(Error::InvalidFormat(format!(
                "Failed to parse .model line '{}': {:?}",
                s_without_comment, e
            ))),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidFormat(format!(
                "Incomplete parse for .model line: '{}'",
                s_without_comment
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_npn_model() {
        let input = ".model MOD1 NPN(BF=50 IS=1e-13 VBF=50)";
        let model = input.parse::<Model>().unwrap();

        assert_eq!(model.name, "MOD1");
        assert_eq!(model.model_type, ModelType::NpnBjt);
        assert_eq!(model.parameters.len(), 3);
        assert_eq!(model.parameters["BF"], 50.0);
        assert_eq!(model.parameters["IS"], 1e-13);
        assert_eq!(model.parameters["VBF"], 50.0);
    }

    #[test]
    fn test_parse_diode_model_lowercase_no_dot() {
        let input = "model DMOD D (is=1e-12 rs=0.1 n=1.1)"; // Example parameters
        let model = input.parse::<Model>().unwrap();

        assert_eq!(model.name, "DMOD");
        assert_eq!(model.model_type, ModelType::Diode);
        assert_eq!(model.parameters.len(), 3);
        assert_eq!(model.parameters["is"], 1e-12);
        assert_eq!(model.parameters["rs"], 0.1);
        assert_eq!(model.parameters["n"], 1.1);
    }

    #[test]
    fn test_parse_model_with_comment() {
        let input = ".MODEL QN2N2222 NPN(IS=1E-14 VAF=100 BF=200) % Standard NPN";
        let model = input.parse::<Model>().unwrap();

        assert_eq!(model.name, "QN2N2222");
        assert_eq!(model.model_type, ModelType::NpnBjt);
        assert_eq!(model.parameters.len(), 3);
        assert_eq!(model.parameters["BF"], 200.0);
    }

    #[test]
    fn test_parse_model_no_parameters() {
        // Technically valid SPICE, though unusual
        let input = ".model MyDiode D()";
        let model = input.parse::<Model>().unwrap();
        assert_eq!(model.name, "MyDiode");
        assert_eq!(model.model_type, ModelType::Diode);
        assert!(model.parameters.is_empty());
    }

    #[test]
    fn test_parse_model_messy_spacing() {
        let input = "  .model   MOD2   PNP  (  BF = 100   IS =1e-12 )  ";
        let model = input.parse::<Model>().unwrap();

        assert_eq!(model.name, "MOD2");
        assert_eq!(model.model_type, ModelType::PnpBjt);
        assert_eq!(model.parameters.len(), 2);
        assert_eq!(model.parameters["BF"], 100.0);
        assert_eq!(model.parameters["IS"], 1e-12);
    }

    #[test]
    fn test_invalid_model_missing_type() {
        let input = ".model MOD1 (BF=50)";
        assert!(input.parse::<Model>().is_err());
    }

    #[test]
    fn test_invalid_model_missing_name() {
        let input = ".model NPN (BF=50)";
        assert!(input.parse::<Model>().is_err());
    }

    #[test]
    fn test_invalid_model_bad_parameter_format() {
        let input = ".model MOD1 NPN(BF 50)"; // Missing '='
        assert!(input.parse::<Model>().is_err());
    }

    #[test]
    fn test_invalid_model_bad_parameter_value() {
        let input = ".model MOD1 NPN(BF=50 IS=abc)"; // 'abc' is not a number
        assert!(input.parse::<Model>().is_err());
    }

    #[test]
    fn test_invalid_model_missing_parentheses() {
        let input = ".model MOD1 NPN BF=50";
        assert!(input.parse::<Model>().is_err());
    }

    #[test]
    fn test_unknown_model_type() {
        // This will parse successfully but store Unknown type if not erroring in FromStr
        let input = ".model MyDevice XYZ (param1=1)";
        let model = input.parse::<Model>().unwrap();
        assert_eq!(model.name, "MyDevice");
        assert_eq!(model.model_type, ModelType::Unknown("XYZ".to_string()));
        assert_eq!(model.parameters.len(), 1);
    }
}
