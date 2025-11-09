use crate::prelude::*;

#[derive(Debug, PartialEq, Clone)]
/// Represents the type of a BJT (Bipolar Junction Transistor).
pub enum BjtType {
    /// NPN BJT.
    NPN,
    /// PNP BJT.
    PNP,
}

#[derive(Debug, Clone)]
/// Represents a BJT (Bipolar Junction Transistor) in a circuit.
pub struct BJT {
    /// Name of the BJT.
    pub name: String,
    /// Collector node of the BJT.
    pub collector: String,
    /// Base node of the BJT.
    pub base: String,
    /// Emitter node of the BJT.
    pub emitter: String,
    /// Value or model name associated with the BJT (optional).
    /// NOTE: SPICE often uses a model name here instead of a simple value.
    ///       The parser now accepts an alphanumeric string, but the `value` field
    ///       remains Option<f64>. This might need adjustment based on how models are handled.
    ///       For now, we attempt to parse it as a value if present.
    pub value: Option<f64>, // Kept as Option<f64> for now
    /// Type of the BJT.
    pub bjt_type: BjtType,
}

impl BJT {
    /// Returns the identifier of the BJT in the format `Q{name}`.
    pub fn identifier(&self) -> String {
        format!("Q{}", self.name)
    }
}

// Nom parser for BJT
pub fn parse_bjt(input: &str) -> IResult<&str, BJT> {
    // Parse the initial 'Q' (case-insensitive)
    let (input, _) = tag_no_case("Q").parse(input)?;

    // Parse the type character (N or P, case-insensitive)
    let (input, type_char) = alt((tag_no_case("N"), tag_no_case("P"))).parse(input)?;
    let bjt_type = match type_char.to_ascii_uppercase().as_str() {
        "N" => BjtType::NPN,
        "P" => BjtType::PNP,
        _ => unreachable!(), // Should be caught by the alt parser
    };

    // Parse the numeric name part
    let (input, name) = alphanumeric1(input)?; // Allows QN123 etc.

    dbg!(name);

    // Parse nodes: collector, base, emitter
    let (input, collector) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, base) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, emitter) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;

    dbg!(&collector, &base, &emitter);

    // Optionally parse the value/model
    let (input, value) = opt(preceded(space1, value_parser)).parse(input)?; // Changed to alphanumeric for model names

    dbg!(&value);

    let bjt = BJT {
        name: name.to_string(),
        collector: collector.to_string(),
        base: base.to_string(),
        emitter: emitter.to_string(),
        value,
        bjt_type,
    };

    Ok((input, bjt))
}

// Updated FromStr using the nom parser
impl FromStr for BJT {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split(['%', '*']).next().unwrap_or("").trim();
        if s_without_comment.is_empty() {
            return Err(Error::InvalidFormat(
                "Empty line after comment removal".to_string(),
            ));
        }

        match all_consuming(parse_bjt).parse(s_without_comment) {
            Ok((_, bjt)) => {
                // Could add checks for node names == "0" if needed, etc.
                Ok(bjt)
            }
            Err(nom::Err::Error(e) | nom::Err::Failure(e)) => Err(Error::InvalidFormat(format!(
                "Failed to parse BJT line '{}': {:?}",
                s_without_comment, e.code
            ))),
            Err(nom::Err::Incomplete(_)) => Err(Error::InvalidFormat(format!(
                "Incomplete parse for BJT line: '{}'",
                s_without_comment
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_npn_bjt_with_value() {
        let bjt_str = "QN1 1 2 0 0.7"; // Emitter to ground
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, "1");
        assert_eq!(bjt.collector, "1");
        assert_eq!(bjt.base, "2");
        assert_eq!(bjt.emitter, "0");
        assert_eq!(bjt.value, Some(0.7));
        assert_eq!(bjt.bjt_type, BjtType::NPN);
        assert_eq!(bjt.identifier(), "Q1");
    }

    #[test]
    fn test_parse_pnp_bjt_with_value() {
        let bjt_str = "QP1 4 5 6 0.8";
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, "1");
        assert_eq!(bjt.collector, "4");
        assert_eq!(bjt.base, "5");
        assert_eq!(bjt.emitter, "6");
        assert_eq!(bjt.value, Some(0.8));
        assert_eq!(bjt.bjt_type, BjtType::PNP);
        assert_eq!(bjt.identifier(), "Q1");
    }

    #[test]
    fn test_parse_bjt_without_value() {
        let bjt_str = "qN2 C B E"; // Case-insensitive, symbolic nodes
        let bjt = bjt_str.parse::<BJT>().unwrap();

        assert_eq!(bjt.name, "2");
        assert_eq!(bjt.collector, "C");
        assert_eq!(bjt.base, "B");
        assert_eq!(bjt.emitter, "E");
        assert_eq!(bjt.value, None);
        assert_eq!(bjt.bjt_type, BjtType::NPN);
        assert_eq!(bjt.identifier(), "Q2");
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "Qp10 coll base emit * My PNP";
        let bjt = s.parse::<BJT>().unwrap();
        assert_eq!(bjt.name, "10");
        assert_eq!(bjt.bjt_type, BjtType::PNP);
        assert_eq!(bjt.value, None);
    }

    #[test]
    fn test_invalid_bjt_format_parts() {
        let bjt_str = "QN1 1 2"; // Missing emitter node
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_bjt_format_extra_parts() {
        let bjt_str = "QN1 1 2 0 0.7 Extra";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err()); // Due to all_consuming
    }

    #[test]
    fn test_invalid_bjt_type() {
        let bjt_str = "QX1 1 2 3"; // Invalid type 'X'
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_identifier_no_type_or_name() {
        let bjt_str = "Q 1 2 3";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_identifier_no_name() {
        let bjt_str = "QN 1 2 3";
        let result = bjt_str.parse::<BJT>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 2 3 100";
        let result = s.parse::<BJT>();
        assert!(result.is_err());
    }
}
