use faer::c64;
use log::info;
use polars::prelude::*;
use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs::File;
use std::path::Path;

/// Ensures the filename ends with `.parquet`
fn ensure_parquet_extension(filename: &str) -> String {
    let path = Path::new(filename);
    if path.extension().and_then(|e| e.to_str()) == Some("parquet") {
        filename.to_string()
    } else {
        format!("{filename}.parquet")
    }
}

/// Writes a single operating point result (`HashMap`<String, f64>) to a Parquet file.
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

    info!("Saved OP results to {filename}");
    Ok(())
}

/// Writes DC sweep results (Vec<`HashMap`<String, f64>>) to a Parquet file.
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

    info!("Saved DC sweep results to {filename}");
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

    info!("Saved transient results to {filename}");
    Ok(())
}

/// Writes AC sweep results (Vec<HashMap<String, c64>>) to a Parquet file.
///
/// The input is a vector where each entry corresponds to one frequency point.
/// Each map contains complex values per signal name, and should include a
/// "frequency" key whose real part is the frequency in Hertz.
///
/// The output Parquet will contain:
/// - A `frequency` column (f64)
/// - For every other key `K`, two columns: `K_mag` and `K_phase_deg` (both f64)
pub fn write_ac_results_to_parquet(
    data: &[HashMap<String, c64>],
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

    // Ensure stable order and put frequency first if present
    all_headers.sort();
    let mut signal_headers: Vec<String> = all_headers
        .into_iter()
        .filter(|h| h != "frequency")
        .collect();
    signal_headers.sort();

    let mut columns: Vec<polars::prelude::Column> = Vec::new();

    // Frequency column (if present) — extract real part only
    {
        let freq_values: Vec<Option<f64>> = data
            .iter()
            .map(|row| row.get("frequency").map(|v| v.re))
            .collect();
        // Include frequency even if all None — remains a valid nullable column
        columns.push(Series::new("frequency".into(), freq_values).into_column());
    }

    // For each other header, create magnitude and phase columns
    for header in signal_headers {
        let mag_name = format!("{}_mag", header);
        let phase_name = format!("{}_phase_deg", header);

        let (mag_values, phase_values): (Vec<Option<f64>>, Vec<Option<f64>>) = data
            .iter()
            .map(|row| {
                row.get(&header).map(|v| {
                    let mag = (v.re * v.re + v.im * v.im).sqrt();
                    let phase = v.im.atan2(v.re) * 180.0 / PI;
                    (mag, phase)
                })
            })
            .map(|opt| match opt {
                Some((m, p)) => (Some(m), Some(p)),
                None => (None, None),
            })
            .unzip();

        columns.push(Series::new(mag_name.into(), mag_values).into_column());
        columns.push(Series::new(phase_name.into(), phase_values).into_column());
    }

    let mut df = DataFrame::new(columns)?;
    let mut file = File::create(&filename).map_err(PolarsError::from)?;
    ParquetWriter::new(&mut file).finish(&mut df)?;

    info!("Saved AC sweep results to {filename}");
    Ok(())
}
