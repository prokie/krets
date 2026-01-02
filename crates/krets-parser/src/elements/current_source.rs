use crate::prelude::*;

#[derive(Debug, Clone)]
/// Represents a current source in a circuit.
pub struct CurrentSource {
    /// The name of the current source.
    pub name: String,
    /// The value of the current source in Amperes.
    pub value: f64,
    /// The positive node of the current source.
    pub plus: String,
    /// The negative node of the current source.
    pub minus: String,
}

impl CurrentSource {
    pub fn identifier(&self) -> String {
        format!("I{}", self.name)
    }
}

pub fn parse_current_source(input: &str) -> IResult<&str, CurrentSource> {
    let (input, _) = tag_no_case("I").parse(input)?;
    let (input, name) = alphanumeric_or_underscore1.parse(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;

    let current_source = CurrentSource {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
    };

    Ok((input, current_source))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_current_source() {
        let current_source_str = "I1 1 0 0.001";
        let (_, current_source) = parse_current_source(current_source_str).unwrap();

        assert_eq!(current_source.name, "1");
        assert_eq!(current_source.plus, "1");
        assert_eq!(current_source.minus, "0");
        assert_eq!(current_source.value, 0.001);
    }

    #[test]
    fn test_parse_with_comment() {
        let s = "I2 5 3 1.5";
        let (_, source) = parse_current_source(s).unwrap();
        assert_eq!(source.name, "2");
        assert_eq!(source.value, 1.5);
    }

    #[test]
    fn test_parse_lowercase_identifier() {
        let s = "i5 vdd gnd 10";
        let (_, source) = parse_current_source(s).unwrap();
        assert_eq!(source.name, "5");
        assert_eq!(source.plus, "vdd");
    }

    #[test]
    fn test_invalid_current_source_format() {
        let current_source_str = "I1 1 0";
        let result = parse_current_source(current_source_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_too_many_parts() {
        // The parser should parse "I1 1 0 1.0" and leave "G2" unparsed
        let input = "I1 1 0 1.0 G2";
        let (remainder, _) = parse_current_source(input).unwrap();
        // The remainder should be " G2" (with leading space)
        assert!(!remainder.is_empty());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "V1 1 0 1.0";
        let result = parse_current_source(s);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_current_source_name() {
        let current_source_str = "I 1 0 0.001";
        let result = parse_current_source(current_source_str);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_current_source_value() {
        let current_source_str = "I1 1 0 abc";
        let result = parse_current_source(current_source_str);
        assert!(result.is_err());
    }
}
