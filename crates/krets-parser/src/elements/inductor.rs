use crate::prelude::*;

#[derive(Debug, Clone)]
/// Represents an inductor in a circuit.
pub struct Inductor {
    /// Name of the inductor.
    pub name: String,
    /// Value of the inductor in Henries.
    pub value: f64,
    /// Positive node of the inductor.
    pub plus: String,
    /// Negative node of the inductor.
    pub minus: String,
}

impl Inductor {
    pub fn identifier(&self) -> String {
        format!("L{}", self.name)
    }
}

pub fn parse_inductor(input: &str) -> IResult<&str, Inductor> {
    let (input, _) = tag_no_case("L").parse(input)?;
    let (input, name) = alphanumeric_or_underscore1(input)?;
    let (input, plus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, minus) = preceded(space1, alphanumeric_or_underscore1).parse(input)?;
    let (input, value) = preceded(space1, value_parser).parse(input)?;

    let inductor = Inductor {
        name: name.to_string(),
        plus: plus.to_string(),
        minus: minus.to_string(),
        value,
    };

    Ok((input, inductor))
}

impl FromStr for Inductor {
    type Err = crate::prelude::Error;

    fn from_str(s: &str) -> Result<Self> {
        let s_without_comment = s.split('%').next().unwrap_or("").trim();
        let (_, inductor) = all_consuming(parse_inductor)
            .parse(s_without_comment)
            .map_err(|e| Error::InvalidFormat(e.to_string()))?;

        Ok(inductor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inductor() {
        let inductor_str = "L1 1 0 0.001";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, "1");
        assert_eq!(inductor.plus, "1");
        assert_eq!(inductor.minus, "0");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_parse_inductor_with_comment() {
        let inductor_str = "L1 1 0 0.001 % This is a comment";
        let inductor = inductor_str.parse::<Inductor>().unwrap();

        assert_eq!(inductor.name, "1");
        assert_eq!(inductor.value, 0.001);
    }

    #[test]
    fn test_parse_lowercase_and_scientific() {
        let s = "l2 vcc out 1e-6";
        let inductor = s.parse::<Inductor>().unwrap();
        assert_eq!(inductor.name, "2");
        assert_eq!(inductor.value, 1e-6);
    }

    #[test]
    fn test_invalid_inductor_format() {
        let inductor_str = "L1 1 0";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_name() {
        let inductor_str = "L 1 0 0.001";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_prefix() {
        let s = "R1 1 0 100";
        let result = s.parse::<Inductor>();
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_inductor_value() {
        let inductor_str = "L1 1 0 abc";
        let result = inductor_str.parse::<Inductor>();
        assert!(result.is_err());
    }
}
