use polars::prelude::*;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Ensures the filename ends with `.parquet`
fn ensure_parquet_extension(filename: &str) -> String {
    let path = Path::new(filename);
    if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
        filename.to_string()
    } else {
        format!("{}.parquet", filename)
    }
}

/// Writes a single operating point result (HashMap<String, f64>) to a Parquet file.
pub fn write_op_results_to_parquet(
    data: &HashMap<String, f64>,
    filename: &str,
) -> Result<(), PolarsError> {
    let filename = ensure_parquet_extension(filename);

    let names: Vec<String> = data.keys().cloned().collect();
    let values: Vec<f64> = data.values().copied().collect();

    let mut df = df![
        "name" => names,
        "value" => values,
    ]?;

    let mut file = File::create(&filename).map_err(PolarsError::from)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    println!("Saved OP results to {}", filename);
    Ok(())
}

/// Writes DC sweep results (Vec<HashMap<String, f64>>) to a Parquet file.
pub fn write_dc_results_to_parquet(
    data: &[HashMap<String, f64>],
    filename: &str,
) -> Result<(), PolarsError> {
    if data.is_empty() {
        return Ok(());
    }

    let filename = ensure_parquet_extension(filename);

    // Get all unique column names from all steps and sort them
    let mut all_headers = data
        .iter()
        .flat_map(|row| row.keys().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    all_headers.sort();

    // Create columns
    let mut columns = Vec::new();
    for header in &all_headers {
        let values: Vec<Option<f64>> = data.iter().map(|row| row.get(header).copied()).collect();
        let series = Series::new(header.as_str().into(), values);
        columns.push(series.into_column());
    }

    let mut df = DataFrame::new(columns)?;

    let mut file = File::create(&filename).map_err(PolarsError::from)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    println!("Saved DC sweep results to {}", filename);
    Ok(())
}
