use nom::{
    IResult, Parser,
    bytes::complete::{is_not, take_while1},
    combinator::map_res,
};

use crate::prelude::*;

/// Parses a SPICE-style numeric value string with metric suffixes.
///
/// This function handles standard floating-point numbers (including scientific notation like `1e-6`)
/// as well as common SPICE suffixes for magnitudes (case-insensitive).
///
/// # Supported Suffixes
/// - `F`: femto (1e-15)
/// - `P`: pico (1e-12)
/// - `N`: nano (1e-9)
/// - `U`: micro (1e-6)
/// - `M`: milli (1e-3)
/// - `K`: kilo (1e3)
/// - `MEG`: mega (1e6)
/// - `G`: giga (1e9)
/// - `T`: tera (1e12)
///
/// # Arguments
/// - `s`: The string slice to parse (e.g., "1.5k", "10u", "1e-6").
///
/// # Returns
/// - A `Result<f64>` containing the parsed floating-point number, or an `Error`.
pub fn parse_value(s: &str) -> Result<f64> {
    let s_upper = s.to_uppercase();

    // Check for a known suffix first. If no suffix is found, the whole string is treated as the number.
    let (num_part_str, multiplier) = if s_upper.ends_with("MEG") {
        // "MEG" is a special 3-character case.
        (&s_upper[..s_upper.len() - 3], 1e6)
    } else if let Some(last_char) = s_upper.chars().last() {
        // Check for single-character suffixes.
        match last_char {
            'F' => (&s_upper[..s_upper.len() - 1], 1e-15),
            'P' => (&s_upper[..s_upper.len() - 1], 1e-12),
            'N' => (&s_upper[..s_upper.len() - 1], 1e-9),
            'U' => (&s_upper[..s_upper.len() - 1], 1e-6),
            'M' => (&s_upper[..s_upper.len() - 1], 1e-3),
            'K' => (&s_upper[..s_upper.len() - 1], 1e3),
            'G' => (&s_upper[..s_upper.len() - 1], 1e9),
            'T' => (&s_upper[..s_upper.len() - 1], 1e12),
            // If the last character is not a known suffix, assume the whole string is the number.
            _ => (s_upper.as_str(), 1.0),
        }
    } else {
        // Handle empty string case.
        (s_upper.as_str(), 1.0)
    };

    // `f64::parse` handles standard float formats, including scientific notation.
    let base_val: f64 = num_part_str
        .parse()
        .map_err(|_| Error::InvalidFloatValue(format!("Invalid numeric value '{}'", s)))?;

    Ok(base_val * multiplier)
}

/// Parses a string consisting of alphanumeric characters and underscores.
pub fn alphanumeric_or_underscore1(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_ascii_alphanumeric() || c == '_').parse(input)
}

/// A nom parser that recognizes a value token and parses it using our custom logic.
pub fn value_parser(input: &str) -> IResult<&str, f64> {
    // 1. Recognize a token (any sequence of chars that isn't a space or parenthesis).
    let token_parser = is_not(" \t\r\n()");

    // 2. Apply your custom parsing function to the recognized token.
    map_res(token_parser, parse_value).parse(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_parser() {
        let epsilon = 1e-15; // Use a smaller epsilon for f64 comparisons
        assert!((parse_value("1.5k").unwrap() - 1500.0).abs() < epsilon);
        assert!((parse_value("10u").unwrap() - 10e-6).abs() < epsilon);
        assert!((parse_value("22n").unwrap() - 22e-9).abs() < epsilon);
        assert!((parse_value("1.2p").unwrap() - 1.2e-12).abs() < epsilon);
        assert!((parse_value("3MEG").unwrap() - 3e6).abs() < epsilon);
        assert!((parse_value("100").unwrap() - 100.0).abs() < epsilon);

        // Test for scientific notation which was previously failing.
        assert!((parse_value("1e-6").unwrap() - 1e-6).abs() < epsilon);
        assert!((parse_value("1.23E-9").unwrap() - 1.23e-9).abs() < epsilon);

        assert!(parse_value("1.5x").is_err());
        assert!(parse_value("garbage").is_err());
    }
}
