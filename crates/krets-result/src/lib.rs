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

    // Create a vector of Series, where each Series is a new column.
    let series: Vec<Series> = data
        .iter()
        // For each (key, value) pair...
        .map(|(name, value)| {
            // Create a Series. The 'name' is the column header.
            // The value is wrapped in a slice `&[*value]` to create a column with a single row.
            Series::new(name.into(), &[*value])
        })
        .collect();

    let mut columns = vec![];
    for serie in series {
        columns.push(serie.into_column());
    }

    // Create a DataFrame from the vector of columns.
    let mut df = DataFrame::new(columns)?;

    // Write the DataFrame to the Parquet file.
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

pub fn write_tran_results_to_parquet(
    data: &[HashMap<String, f64>],
    filename: &str,
) -> Result<(), PolarsError> {
    if data.is_empty() {
        return Ok(());
    }

    let filename = ensure_parquet_extension(filename);

    // Collect all unique headers
    let mut all_headers = data
        .iter()
        .flat_map(|row| row.keys().cloned())
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    // If a "time" column exists, ensure it's first
    all_headers.sort();
    if let Some(pos) = all_headers.iter().position(|h| h == "time") {
        all_headers.remove(pos);
        all_headers.insert(0, "time".to_string());
    }

    // Build columns
    let mut columns = Vec::with_capacity(all_headers.len());
    for header in &all_headers {
        let values: Vec<Option<f64>> = data.iter().map(|row| row.get(header).copied()).collect();
        let series = Series::new(header.to_string().into(), values);
        columns.push(series.into_column());
    }

    let mut df = DataFrame::new(columns)?;
    let mut file = File::create(&filename).map_err(PolarsError::from)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    println!("Saved transient results to {}", filename);
    Ok(())
}
