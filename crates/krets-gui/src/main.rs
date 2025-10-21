use arrow::record_batch::RecordBatch;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Legend, Line, Plot, PlotPoint, PlotPoints, Text};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::collections::HashSet;
use std::{fs, path::PathBuf};

/// Represents an entry in the directory listing.
struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
}

/// Holds the data loaded from a Parquet file for display.
struct TableData {
    /// The column names.
    headers: Vec<String>,
    /// The data itself, held as an Arrow RecordBatch.
    batch: RecordBatch,
}

struct KretsApp {
    current_path: PathBuf,
    entries: Vec<DirectoryEntry>,
    error_message: Option<String>,
    file_to_load: Option<PathBuf>,
    table_data: Option<TableData>,
    selection: HashSet<usize>,
}

impl Default for KretsApp {
    fn default() -> Self {
        // Start in the current working directory.
        let mut app = Self {
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            entries: Vec::new(),
            error_message: None,
            file_to_load: None,
            table_data: None,
            selection: HashSet::new(),
        };
        app.refresh_entries();
        app
    }
}

impl eframe::App for KretsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut path_to_navigate = None;

        egui::SidePanel::left("file_panel").show(ctx, |ui| {
            ui.heading("Circuit File Explorer");
            ui.separator();

            ui.horizontal(|ui| {
                if ui.button("⬆ Up").clicked() {
                    if let Some(parent) = self.current_path.parent() {
                        path_to_navigate = Some(parent.to_path_buf());
                    }
                }
                ui.label(format!("Path: {}", self.current_path.display()));
            });

            ui.separator();

            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in &self.entries {
                        let icon = if entry.is_directory { "📁" } else { "📄" };
                        let file_name =
                            entry.path.file_name().unwrap_or_default().to_string_lossy();

                        // Make all entries buttons to handle clicks.
                        if ui.button(format!("{icon} {file_name}")).clicked() {
                            if entry.is_directory {
                                // Navigate into the directory if it's a directory.
                                path_to_navigate = Some(entry.path.clone());
                            } else {
                                // If it's a file, check if it's a parquet file and set it for loading.
                                if entry.path.extension().map_or(false, |ext| ext == "parquet") {
                                    self.file_to_load = Some(entry.path.clone());
                                }
                            }
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Data Viewer");

            // Display column statistics (Min/Max) instead of the raw data table.
            if let Some(data) = &self.table_data {
                // Use a TableBuilder to display the column stats.
                let table = TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .columns(Column::auto(), 4)
                    .sense(egui::Sense::click());

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Name");
                        });
                        header.col(|ui| {
                            ui.strong("Min");
                        });
                        header.col(|ui| {
                            ui.strong("Max");
                        });
                        header.col(|ui| {
                            ui.strong("Select");
                        });
                    })
                    .body(|mut body| {
                        // Iterate over the *columns* in the batch.
                        // Each column will be a *row* in our new table.
                        for (index, array) in data.batch.columns().iter().enumerate() {
                            let column_name = &data.headers[index];

                            // Get the min/max statistics for this array
                            let (min_str, max_str) = get_col_stats(array);

                            body.row(18.0, |mut row| {
                                // First cell is the column name
                                row.col(|ui| {
                                    ui.label(column_name);
                                });
                                // Second cell is the min value
                                row.col(|ui| {
                                    ui.label(min_str);
                                });
                                // Third cell is the max value
                                row.col(|ui| {
                                    ui.label(max_str);
                                });

                                row.col(|ui| {
                                    // Check if this row's index is already in the selection
                                    let mut is_checked = self.selection.contains(&index);

                                    // Create a checkbox. `ui.checkbox` will modify `is_checked` if clicked.
                                    let response = ui.checkbox(&mut is_checked, ""); // Use an empty label

                                    // If the checkbox was clicked, update the HashSet
                                    if response.changed() {
                                        if is_checked {
                                            // If it's now checked, add the index
                                            self.selection.insert(index);
                                        } else {
                                            // If it's now unchecked, remove the index
                                            self.selection.remove(&index);
                                        }
                                    }
                                });
                            });
                        }
                    });
            } else {
                ui.label("Select a .parquet file from the explorer to view its data.");
            }

            ui.separator();
            ui.heading("Plot Viewer");
            let my_plot = Plot::new("My Plot").legend(Legend::default());
            my_plot.show(ui, |plot_ui| {
                // Only plot if we have data and *at least* two columns are selected
                if let Some(data) = &self.table_data {
                    if self.selection.len() >= 2 {
                        // Get the selected indices.
                        // We sort them so the X-axis is deterministic
                        // (lowest index will be X, all others will be Y).
                        let mut indices: Vec<usize> = self.selection.iter().copied().collect();
                        indices.sort();

                        let idx_x = indices[0];
                        let name_x = &data.headers[idx_x];
                        let col_x_arr = &data.batch.columns()[idx_x];

                        // Try to get the X-axis data
                        if let Some(x_vals) = get_column_as_f64(col_x_arr) {
                            // Now, iterate over all *other* selected columns and plot them as Y
                            let mut plotted_anything = false;
                            for &idx_y in &indices[1..] {
                                let name_y = &data.headers[idx_y];
                                let col_y_arr = &data.batch.columns()[idx_y];

                                // Try to get the Y-axis data
                                if let Some(y_vals) = get_column_as_f64(col_y_arr) {
                                    let line_name = format!("{} (Y) vs. {} (X)", name_y, name_x);

                                    // Combine the X and Y vectors into PlotPoints
                                    let points: PlotPoints = x_vals
                                        .iter()
                                        .zip(y_vals.iter())
                                        .map(|(&x, &y)| [x, y])
                                        .collect();

                                    plot_ui.line(Line::new(line_name, points));
                                    plotted_anything = true;
                                }
                                // If a specific Y column isn't numeric, we just skip it.
                            }

                            if !plotted_anything {
                                // plot_ui.text("X-axis is plottable, but no selected Y-axis columns are plottable.");
                            }
                        } else {
                            // This handles cases where the X-axis column is not numeric
                            // plot_ui.text("The first selected column (lowest index) is not plottable.");
                        }
                    } else {
                        // This handles 0 or 1 selected columns
                        // plot_ui.text("Select at least two numeric columns to plot (first selected = X, others = Y).");
                    }
                } else {
                    // This handles the case before a file is loaded
                    // plot_ui.text(Text::new(
                    //     "Load a Parquet file and select columns to plot.",
                    //     PlotPoint::new(-3.0, -3.0),
                    //     "here",
                    // ));
                }
            });
        });

        if let Some(new_path) = path_to_navigate {
            self.current_path = new_path;
            self.refresh_entries();
        }

        // If a file was selected for loading, process it now.
        if let Some(path) = self.file_to_load.take() {
            self.load_parquet_file(&path);
        }
    }
}

impl KretsApp {
    fn refresh_entries(&mut self) {
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                let mut dir_entries = Vec::new();
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    let is_directory = path.is_dir();
                    dir_entries.push(DirectoryEntry { path, is_directory });
                }
                // Sort directories first, then files, all alphabetically.
                dir_entries.sort_by(|a, b| {
                    (b.is_directory, a.path.file_name()).cmp(&(a.is_directory, b.path.file_name()))
                });
                self.entries = dir_entries;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!("Failed to read directory: {}", e));
            }
        }
    }

    /// Loads data from a specified Parquet file into the app's state.
    fn load_parquet_file(&mut self, path: &PathBuf) {
        self.table_data = None; // Clear previous data
        self.error_message = None;
        self.selection.clear();

        match fs::File::open(path) {
            Ok(file) => {
                match ParquetRecordBatchReaderBuilder::try_new(file) {
                    Ok(builder) => match builder.build() {
                        Ok(mut reader) => {
                            // We'll load the first batch for simplicity.
                            if let Some(Ok(batch)) = reader.next() {
                                let headers = batch
                                    .schema()
                                    .fields()
                                    .iter()
                                    .map(|field| field.name().clone())
                                    .collect();

                                self.table_data = Some(TableData { headers, batch });
                            } else {
                                self.error_message =
                                    Some("Parquet file is empty or corrupt.".to_string());
                            }
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to read Parquet: {}", e))
                        }
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Failed to build Parquet reader: {}", e))
                    }
                }
            }
            Err(e) => self.error_message = Some(format!("Failed to open file: {}", e)),
        }
    }
}

/// Helper to get min/max stats for an Arrow array as strings.
fn get_col_stats(array: &arrow::array::ArrayRef) -> (String, String) {
    // Local imports for this helper function
    use arrow::array::*;
    use arrow::compute::kernels::aggregate::{max, min};

    // Helper to format Option<T> where T: ToString
    fn format_opt<T: ToString>(opt: Option<T>) -> String {
        opt.map(|v| v.to_string())
            .unwrap_or_else(|| "NULL".to_string())
    }

    match array.data_type() {
        arrow::datatypes::DataType::Int8 => {
            let arr = array.as_any().downcast_ref::<Int8Array>().unwrap();
            (format_opt(min(arr)), format_opt(max(arr)))
        }
        arrow::datatypes::DataType::Int16 => {
            let arr = array.as_any().downcast_ref::<Int16Array>().unwrap();
            (format_opt(min(arr)), format_opt(max(arr)))
        }
        arrow::datatypes::DataType::Int32 => {
            let arr = array.as_any().downcast_ref::<Int32Array>().unwrap();
            (format_opt(min(arr)), format_opt(max(arr)))
        }
        arrow::datatypes::DataType::Int64 => {
            let arr = array.as_any().downcast_ref::<Int64Array>().unwrap();
            (format_opt(min(arr)), format_opt(max(arr)))
        }
        arrow::datatypes::DataType::Float32 => {
            let arr = array.as_any().downcast_ref::<Float32Array>().unwrap();
            let min_str = min(arr)
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "NULL".to_string());
            let max_str = max(arr)
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "NULL".to_string());
            (min_str, max_str)
        }
        arrow::datatypes::DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            let min_str = min(arr)
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "NULL".to_string());
            let max_str = max(arr)
                .map(|v| format!("{:.4}", v))
                .unwrap_or_else(|| "NULL".to_string());
            (min_str, max_str)
        }

        _ => ("N/A".to_string(), "N/A".to_string()),
    }
}

/// Helper to get all values from a numeric Arrow array as `Vec<f64>`.
/// Returns `None` if the array is not a supported numeric type.
/// Nulls in the array are converted to `f64::NAN`.
fn get_column_as_f64(array: &arrow::array::ArrayRef) -> Option<Vec<f64>> {
    use arrow::array::*;

    // Macro to simplify conversion for different numeric types
    macro_rules! convert_numeric_array {
        ($arr_type:ty) => {{
            let arr = array.as_any().downcast_ref::<$arr_type>().unwrap();
            // Iterate, convert nulls to NAN, and cast all values to f64
            Some(
                arr.iter()
                    .map(|v| v.map_or(f64::NAN, |val| val as f64))
                    .collect(),
            )
        }};
    }

    match array.data_type() {
        arrow::datatypes::DataType::Int8 => convert_numeric_array!(Int8Array),
        arrow::datatypes::DataType::Int16 => convert_numeric_array!(Int16Array),
        arrow::datatypes::DataType::Int32 => convert_numeric_array!(Int32Array),
        arrow::datatypes::DataType::Int64 => convert_numeric_array!(Int64Array),
        arrow::datatypes::DataType::UInt8 => convert_numeric_array!(UInt8Array),
        arrow::datatypes::DataType::UInt16 => convert_numeric_array!(UInt16Array),
        arrow::datatypes::DataType::UInt32 => convert_numeric_array!(UInt32Array),
        arrow::datatypes::DataType::UInt64 => convert_numeric_array!(UInt64Array),
        arrow::datatypes::DataType::Float32 => convert_numeric_array!(Float32Array),
        arrow::datatypes::DataType::Float64 => convert_numeric_array!(Float64Array),
        // Other types are not considered plottable
        _ => None,
    }
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Krets - Parquet Viewer",
        options,
        Box::new(|_cc| Ok(Box::<KretsApp>::default())),
    )
}
