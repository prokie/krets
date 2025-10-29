pub mod diode;
pub mod nmosfet;
pub mod pmosfet;

use crate::{models::diode::DiodeModel, prelude::*};
use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{space0, space1},
    combinator::map,
    multi::many0,
    sequence::{delimited, preceded},
};

#[derive(Debug, PartialEq, Clone)]
/// Enum representing the different types of devices supported by the .model card.
pub enum Model {
    Diode(diode::DiodeModel),       // D
    NMosfet(nmosfet::NMosfetModel), // NMOSFET
    PMosfet(pmosfet::PMosfetModel), // PMOSFET
}

pub trait ModelTrait {
    fn apply_model_parameters(&mut self, parameters: &HashMap<String, f64>);
}

/// Parses a list of parameters like (KEY=VALUE KEY2=VALUE2 ...)
fn parse_parameters(input: &str) -> IResult<&str, HashMap<String, f64>> {
    delimited(
        preceded(space0, tag("(")),
        map(many0(preceded(space0, parse_key_value)), |vec| {
            vec.into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<HashMap<String, f64>>()
        }),
        preceded(space0, tag(")")),
    )
    .parse(input)
}

pub fn parse_model_variant(input: &str) -> IResult<&str, Model> {
    let (input, _) = preceded(tag_no_case(".model"), space1).parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, _) = space1(input)?;

    alt((
        map((tag("NMOS"), parse_parameters), move |(_, parameters)| {
            let mut nmosfet_model = nmosfet::NMosfetModel {
                name: name.to_string(),
                ..Default::default()
            };
            nmosfet_model.apply_model_parameters(&parameters);
            Model::NMosfet(nmosfet_model)
        }),
        map((tag("PMOS"), parse_parameters), move |(_, parameters)| {
            let mut pmosfet_model = pmosfet::PMosfetModel {
                name: name.to_string(),
                ..Default::default()
            };
            pmosfet_model.apply_model_parameters(&parameters);
            Model::PMosfet(pmosfet_model)
        }),
        map((tag("D"), parse_parameters), move |(_, parameters)| {
            let mut diode_model = DiodeModel {
                name: name.to_string(),
                ..Default::default()
            };
            diode_model.apply_model_parameters(&parameters);
            Model::Diode(diode_model)
        }),
    ))
    .parse(input)
}

pub fn parse_model(input: &str) -> Result<Model> {
    let input_without_comment = input.split('%').next().unwrap_or("").trim();
    let (_, model) = parse_model_variant
        .parse(input_without_comment)
        .map_err(|e| Error::InvalidFormat(e.to_string()))?;

    Ok(model)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diode() {
        let input = ".model DMOD D (is=1e-12 rs=0.1 n=1.1)";
        let model = parse_model(input).unwrap();
        match model {
            Model::Diode(diode_model) => {
                assert_eq!(diode_model.name, "DMOD");
                assert_eq!(diode_model.saturation_current, 1e-12);
                assert_eq!(diode_model.parasitic_resistance, 0.1);
                assert_eq!(diode_model.emission_coefficient, 1.1);
            }
            _ => panic!("Expected Diode model"),
        }
    }

    #[test]
    fn test_parse_nmos() {
        let input = ".model NMOS1 NMOS (kp=120u vto=1.2 lambda=0.02)";
        let model = parse_model(input).unwrap();

        match model {
            Model::NMosfet(nmos_model) => {
                assert_eq!(nmos_model.name, "NMOS1");
                assert!((nmos_model.process_transconductance - 120e-6).abs() < 1e-3);
                assert!((nmos_model.voltage_threshold - 1.2).abs() < 1e-3);
                assert!((nmos_model.channel_length_modulation - 0.02).abs() < 1e-3);
            }
            _ => panic!("Expected NMOSFET model"),
        }
    }

    #[test]
    fn test_parse_pmos() {
        let input = ".model PMOS1 PMOS (kp=50u vto=-1.0 lambda=0.01)";
        let model = parse_model(input).unwrap();

        match model {
            Model::PMosfet(pmos_model) => {
                assert_eq!(pmos_model.name, "PMOS1");
                assert!((pmos_model.process_transconductance - 50e-6).abs() < 1e-3);
                assert!((pmos_model.voltage_threshold + 1.0).abs() < 1e-3);
                assert!((pmos_model.channel_length_modulation - 0.01).abs() < 1e-3);
            }
            _ => panic!("Expected PMOSFET model"),
        }
    }

    #[test]
    fn test_parse_model_no_parameters() {
        // Technically valid SPICE, though unusual
        let input = ".model MyDiode D()";
        let model = parse_model(input).unwrap();
        match model {
            Model::Diode(diode_model) => {
                assert_eq!(diode_model.name, "MyDiode");
            }
            _ => panic!("Expected Diode model"),
        }
    }

    #[test]
    fn test_invalid_model_missing_type() {
        let input = ".model MOD1 (BF=50)";
        assert!(parse_model(input).is_err());
    }

    #[test]
    fn test_invalid_model_missing_name() {
        let input = ".model NPN (BF=50)";
        assert!(parse_model(input).is_err());
    }

    #[test]
    fn test_invalid_model_bad_parameter_format() {
        let input = ".model MOD1 NPN(BF 50)"; // Missing '='
        assert!(parse_model(input).is_err());
    }

    #[test]
    fn test_invalid_model_bad_parameter_value() {
        let input = ".model MOD1 NPN(BF=50 IS=abc)"; // 'abc' is not a number
        assert!(parse_model(input).is_err());
    }

    #[test]
    fn test_invalid_model_missing_parentheses() {
        let input = ".model MOD1 NPN BF=50";
        assert!(parse_model(input).is_err());
    }
}
