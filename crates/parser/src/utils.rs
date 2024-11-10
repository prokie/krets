use crate::prelude::*;
use std::str::FromStr;

/// Parses a string representing a value with an optional unit suffix into an `f64`.
///
/// The function supports the following unit suffixes (case insensitive):
///
/// * `T` - 1e12
/// * `G` - 1e9
/// * `MEG` - 1e6
/// * `X` - 1e6
/// * `K` - 1e3
/// * `M` - 1e-3
/// * `U` - 1e-6
/// * `N` - 1e-9
/// * `P` - 1e-12
/// * `F` - 1e-15
///
/// If no unit suffix is found, the function attempts to parse the value directly as an `f64`.
///
/// # Arguments
///
/// * `value` - A string slice that holds the value to be parsed.
///
/// # Returns
///
/// * `Ok(f64)` - The parsed value as an `f64` if the parsing is successful.
/// * `Err(Error)` - An error if the parsing fails.
///
/// # Errors
///
/// This function will return an error in the following cases:
///
/// * [`Error::InvalidFloatValue`] - If the numeric part of the value cannot be parsed as an `f64`.
///
/// # Examples
///
/// ```
/// use crate::parser::utils::parse_value;
///
/// assert_eq!(parse_value("1T").unwrap(), 1e12);
/// assert_eq!(parse_value("1.5K").unwrap(), 1.5e3);
/// assert_eq!(parse_value("123.456").unwrap(), 123.456);
/// assert!(parse_value("invalid").is_err());
/// ```
pub fn parse_value(value: &str) -> Result<f64> {
    let value = value.trim().to_uppercase();
    let symbols = [
        ("T", 1e12),
        ("G", 1e9),
        ("MEG", 1e6),
        ("X", 1e6),
        ("K", 1e3),
        ("M", 1e-3),
        ("U", 1e-6),
        ("N", 1e-9),
        ("P", 1e-12),
        ("F", 1e-15),
    ];

    for (symbol, multiplier) in &symbols {
        if value.ends_with(symbol) {
            let number_part = &value[..value.len() - symbol.len()];
            let number = f64::from_str(number_part)
                .map_err(|_| Error::InvalidFloatValue(number_part.to_string()))?;
            return Ok(number * multiplier);
        }
    }

    f64::from_str(&value).map_err(|_| Error::InvalidFloatValue(value.clone()))
}
