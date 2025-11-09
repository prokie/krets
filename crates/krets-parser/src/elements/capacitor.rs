use crate::prelude::*;

#[derive(Debug, Clone)]
/// Represents a capacitor in a circuit.
pub struct Capacitor {
    /// Name of the capacitor.
    pub name: String,
    /// Value of the capacitor.
    pub value: f64,
    /// Positive node of the capacitor.
    pub plus: String,
    /// Negative node of the capacitor.
    pub minus: String,
    /// If the capacitor is G2.
    pub g2: bool,
}

impl Capacitor {
    pub fn identifier(&self) -> String {
        format!("C{}", self.name)
    }
}

pub fn parse_capacitor(input: &str) -> IResult<&str, Capacitor> {
    let (input, _) = tag_no_case("C").parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;
    let (input, g2_opt) = opt(preceded(space1, tag_no_case("G2"))).parse(input)?;

    let capacitor = Capacitor {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
        g2: g2_opt.is_some(),
    };

    Ok((input, capacitor))
}

impl FromStr for Capacitor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();

        let (_, capacitor) = all_consuming(parse_capacitor)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        Ok(capacitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_capacitor() {
        let capacitor_str = "C1 1 0 0.000001";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_group() {
        let capacitor_str = "C1 1 0 0.000001 G2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment() {
        let capacitor_str = "C1 1 0 0.000001 % This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.plus, "1");
        assert_eq!(capacitor.minus, "0");
        assert_eq!(capacitor.value, 0.000001);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_comment_no_space() {
        let capacitor_str = "C1 1 0 1e-6%This is a comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert_eq!(capacitor.value, 1e-6);
        assert!(!capacitor.g2);
    }

    #[test]
    fn test_parse_capacitor_with_g2_and_comment() {
        let capacitor_str = "c2 3 4 10e-9 G2 % comment";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "2");
        assert_eq!(capacitor.value, 10e-9);
        assert!(capacitor.g2);
    }

    #[test]
    fn test_parse_lowercase() {
        let capacitor_str = "c1 1 0 1e-6 g2";
        let capacitor = capacitor_str.parse::<Capacitor>().unwrap();

        assert_eq!(capacitor.name, "1");
        assert!(capacitor.g2);
    }

    #[test]
    fn test_invalid_capacitor_format() {
        let capacitor_str = "C1 1 0";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_name() {
        let capacitor_str = "C 1 0 0.000001";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_identifier_prefix() {
        let capacitor_str = "R1 1 0 100";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_fifth_argument() {
        let capacitor_str = "C1 1 0 1e-6 G3";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_capacitor_value() {
        let capacitor_str = "C1 1 0 abc";
        let result = capacitor_str.parse::<Capacitor>();
        assert!(result.is_err());
    }
}
