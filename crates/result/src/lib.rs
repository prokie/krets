use polars::prelude::*;
use std::collections::HashMap;
use std::fs::File;

/// Writes a simple HashMap<String, f64> to a Parquet file.
///
/// # Arguments
/// * `data` - A HashMap where keys are variable names (e.g., "V1", "V2") and values are floating-point numbers.
/// * `filename` - The output Parquet filename.
///
/// # Example
/// ```rust
/// use result::write_dict_to_parquet;
/// use std::collections::HashMap;
/// let mut data = HashMap::new();
/// data.insert("V1".to_string(), 3.0);
/// data.insert("V2".to_string(), 0.5);
/// write_dict_to_parquet(&data, "simple_results.parquet").unwrap();
/// ```
pub fn write_dict_to_parquet(
    data: &HashMap<String, f64>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Convert HashMap to a Polars DataFrame
    let names: Vec<String> = data.keys().cloned().collect();
    let values: Vec<f64> = data.values().copied().collect();

    let df = df!(
        "name" => names,
        "value"=>  values,
    )?;

    // Write DataFrame to Parquet
    let mut file = File::create(filename)?;
    ParquetWriter::new(&mut file).finish(&mut df.clone())?;

    println!("Saved to {filename}");
    Ok(())
}
