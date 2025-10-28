use arrow::record_batch::RecordBatch;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Legend, Line, Plot, PlotPoints};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use std::collections::HashSet;
use std::{fs, path::PathBuf};

/// Represents an entry in the directory listing.
#[derive(Clone)] // Added Clone
struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
}

/// Holds the data loaded from a Parquet file for display.
struct TableData {
    /// The column names.
    headers: Vec<String>,
    /// The data itself, held as an Arrow `RecordBatch`.
    batch: RecordBatch,
}

struct KretsApp {
    current_path: PathBuf,
    entries: Vec<DirectoryEntry>,
    error_message: Option<String>,
    file_to_load: Option<PathBuf>, // Initial file to load
    table_data: Option<TableData>,
    selection: HashSet<usize>,
    current_loaded_file: Option<PathBuf>,
}

impl KretsApp {
    // Renamed default to new and accept parameters
    fn new(initial_folder_path: PathBuf, initial_result_file: Option<PathBuf>) -> Self {
        let mut app = Self {
            current_path: initial_folder_path
                .canonicalize()
                .unwrap_or(initial_folder_path), // Canonicalize for cleaner display
            entries: Vec::new(),
            error_message: None,
            file_to_load: initial_result_file.clone(), // Set initial file to load
            table_data: None,
            selection: HashSet::new(),
            current_loaded_file: None,
        };
        app.refresh_entries();

        // Immediately try loading the initial file if provided
        if let Some(path) = initial_result_file {
            app.load_parquet_file(&path);
        }

        app
    }
}

impl eframe::App for KretsApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // This will be set by the file explorer UI if navigation is requested.
        let mut path_to_navigate = None;

        egui::SidePanel::left("file_panel").show(ctx, |ui| {
            // Delegate file explorer UI and logic
            path_to_navigate = self.ui_file_explorer(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Delegate central panel UI and logic
            self.ui_central_panel(ui);
        });

        // --- Post-UI Logic ---
        // Handle navigation requested from the side panel
        if let Some(new_path) = path_to_navigate {
            self.current_path = new_path;
            self.refresh_entries();
        }

        // Handle file loading requested from the side panel *after* initial load
        if let Some(path) = self.file_to_load.take() {
            // Only load if it's different from the currently loaded file
            let already_loaded = self
                .current_loaded_file
                .as_ref()
                .map(|p| p == &path)
                .unwrap_or(false);

            if !already_loaded {
                self.load_parquet_file(&path);
            }
        }
    }
}

impl KretsApp {
    /// Renders the file explorer side panel.
    /// Returns an `Option<PathBuf>` if navigation is requested.
    fn ui_file_explorer(&mut self, ui: &mut egui::Ui) -> Option<PathBuf> {
        let mut path_to_navigate = None;

        ui.heading("Circuit File Explorer");
        ui.separator();

        ui.horizontal(|ui| {
            // Disable "Up" button if we are at the root
            let is_at_root = self.current_path.parent().is_none();
            let up_button = ui.add_enabled(!is_at_root, egui::Button::new("⬆ Up"));

            if up_button.clicked()
                && let Some(parent) = self.current_path.parent()
            {
                // Ensure parent exists and canonicalize
                if let Ok(canon_parent) = parent.canonicalize() {
                    path_to_navigate = Some(canon_parent);
                } else {
                    // Fallback if canonicalization fails (e.g., permissions)
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
                for entry in self.entries.clone() {
                    // Clone entry to avoid borrow checker issues with mutable self
                    let icon = if entry.is_directory { "📁" } else { "📄" };
                    let file_name = entry.path.file_name().unwrap_or_default().to_string_lossy();
                    let is_parquet = entry.path.extension().is_some_and(|ext| ext == "parquet");

                    // Only enable button for directories and parquet files
                    let enabled = entry.is_directory || is_parquet;
                    let response =
                        ui.add_enabled(enabled, egui::Button::new(format!("{icon} {file_name}")));

                    if response.clicked() {
                        if entry.is_directory {
                            // Ensure path exists and canonicalize
                            if let Ok(canon_path) = entry.path.canonicalize() {
                                path_to_navigate = Some(canon_path);
                            } else {
                                path_to_navigate = Some(entry.path); // Fallback
                            }
                        } else if is_parquet {
                            // If it's a parquet file, set it for loading.
                            // Clone needed as entry might be invalidated by refresh_entries
                            if let Ok(canon_path) = entry.path.canonicalize() {
                                self.file_to_load = Some(canon_path);
                            } else {
                                self.file_to_load = Some(entry.path); // Fallback
                            }
                        }
                    }
                }
            });
        }

        // Return the navigation request to the main update loop
        path_to_navigate
    }

    /// Renders the central panel, delegating to table and plot methods.
    fn ui_central_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Data Viewer");
        self.ui_stats_table(ui);

        ui.separator();
        ui.heading("Plot Viewer");
        self.ui_plot_viewer(ui);
    }

    /// Renders the column statistics table.
    fn ui_stats_table(&mut self, ui: &mut egui::Ui) {
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
    }

    /// Renders the plot viewer.
    fn ui_plot_viewer(&mut self, ui: &mut egui::Ui) {
        let my_plot = Plot::new("My Plot").legend(Legend::default());
        my_plot.show(ui, |plot_ui| {
            // Only plot if we have data and *at least* two columns are selected
            if let Some(data) = &self.table_data
                && self.selection.len() >= 2
            {
                // Get the selected indices.
                // We sort them so the X-axis is deterministic
                // (lowest index will be X, all others will be Y).
                let mut indices: Vec<usize> = self.selection.iter().copied().collect();
                indices.sort_unstable();

                let idx_x = indices[0];
                let name_x = &data.headers[idx_x];
                let col_x_arr = &data.batch.columns()[idx_x];

                // Try to get the X-axis data
                if let Some(x_vals) = get_column_as_f64(col_x_arr) {
                    // Now, iterate over all *other* selected columns and plot them as Y
                    for &idx_y in &indices[1..] {
                        let name_y = &data.headers[idx_y];
                        let col_y_arr = &data.batch.columns()[idx_y];

                        // Try to get the Y-axis data
                        if let Some(y_vals) = get_column_as_f64(col_y_arr) {
                            let line_name = format!("{name_y} (Y) vs. {name_x} (X)");

                            // Combine the X and Y vectors into PlotPoints
                            // Ensure vectors are the same length before zipping
                            let points: PlotPoints = x_vals
                                .iter()
                                .zip(y_vals.iter())
                                .map(|(&x, &y)| [x, y])
                                .collect();

                            plot_ui.line(Line::new(line_name, points));
                        }
                    }
                }
            }
        });
    }

    fn refresh_entries(&mut self) {
        match fs::read_dir(&self.current_path) {
            Ok(entries) => {
                let mut dir_entries = Vec::new();
                for entry in entries.filter_map(Result::ok) {
                    if let Ok(metadata) = entry.metadata() {
                        let path = entry.path();
                        let is_directory = metadata.is_dir();
                        // Optional: Filter out hidden files/dirs if desired
                        // if !path.file_name().map_or(false, |name| name.to_string_lossy().starts_with('.')) {
                        dir_entries.push(DirectoryEntry { path, is_directory });
                        // }
                    }
                }
                // Sort directories first, then files, all alphabetically.
                dir_entries.sort_by(|a, b| {
                    (b.is_directory, a.path.file_name().unwrap_or_default())
                        .cmp(&(a.is_directory, b.path.file_name().unwrap_or_default()))
                });
                self.entries = dir_entries;
                self.error_message = None;
            }
            Err(e) => {
                self.error_message = Some(format!(
                    "Failed to read directory '{}': {}",
                    self.current_path.display(),
                    e
                ));
                self.entries.clear(); // Clear entries on error
            }
        }
    }

    /// Loads data from a specified Parquet file into the app's state.
    fn load_parquet_file(&mut self, path: &PathBuf) {
        self.table_data = None; // Clear previous data
        self.error_message = None;
        self.selection.clear(); // Clear selection when loading new file

        match fs::File::open(path) {
            Ok(file) => {
                match ParquetRecordBatchReaderBuilder::try_new(file) {
                    Ok(builder) => match builder.build() {
                        Ok(reader) => {
                            // Load all batches into a single Vec for simplicity
                            let batches: Vec<Result<RecordBatch, _>> = reader.collect();
                            let ok_batches: Vec<RecordBatch> =
                                batches.into_iter().filter_map(Result::ok).collect();

                            if ok_batches.is_empty() {
                                self.error_message = Some(
                                    "Parquet file is empty or has no valid batches.".to_string(),
                                );
                                return;
                            }

                            // For simplicity, we'll just display the first batch.
                            // Concatenating batches could be done here if needed.
                            let first_batch = ok_batches[0].clone();

                            let headers = first_batch
                                .schema()
                                .fields()
                                .iter()
                                .map(|field| field.name().clone())
                                .collect();

                            self.table_data = Some(TableData {
                                headers,
                                batch: first_batch,
                            });

                            // Update file_to_load to reflect the currently loaded file path
                            // Canonicalize for consistency if possible
                            let canonical = path.canonicalize().ok().or_else(|| Some(path.clone()));
                            self.file_to_load = canonical.clone();
                            // Record the successfully loaded file so future clicks on the same file do nothing
                            self.current_loaded_file = canonical;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Failed to read Parquet batch: {e}"));
                        }
                    },
                    Err(e) => {
                        self.error_message = Some(format!("Failed to build Parquet reader: {e}"));
                    }
                }
            }
            Err(e) => {
                self.error_message =
                    Some(format!("Failed to open file '{}': {}", path.display(), e))
            }
        }
    }
}

/// Helper to get min/max stats for an Arrow array as strings.
fn get_col_stats(array: &arrow::array::ArrayRef) -> (String, String) {
    use arrow::array::{
        Array, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
    };
    use arrow::compute::kernels::aggregate::{max, min};

    fn format_opt<T: ToString>(opt: Option<T>) -> String {
        opt.map_or_else(|| "NULL".to_string(), |v| v.to_string())
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
            let min_str = min(arr).map_or_else(|| "NULL".to_string(), |v| format!("{v:.4}"));
            let max_str = max(arr).map_or_else(|| "NULL".to_string(), |v| format!("{v:.4}"));
            (min_str, max_str)
        }
        arrow::datatypes::DataType::Float64 => {
            let arr = array.as_any().downcast_ref::<Float64Array>().unwrap();
            let min_str = min(arr).map_or_else(|| "NULL".to_string(), |v| format!("{v:.4}"));
            let max_str = max(arr).map_or_else(|| "NULL".to_string(), |v| format!("{v:.4}"));
            (min_str, max_str)
        }
        _ => ("N/A".to_string(), "N/A".to_string()),
    }
}

/// Helper to get all values from a numeric Arrow array as `Vec<f64>`.
/// Returns `None` if the array is not a supported numeric type.
/// Nulls in the array are converted to `f64::NAN`.
fn get_column_as_f64(array: &arrow::array::ArrayRef) -> Option<Vec<f64>> {
    use arrow::array::{
        Array, Float32Array, Float64Array, Int8Array, Int16Array, Int32Array, Int64Array,
        UInt8Array, UInt16Array, UInt32Array, UInt64Array,
    };

    macro_rules! convert_numeric_array {
        ($arr_type:ty) => {{
            let arr = array.as_any().downcast_ref::<$arr_type>()?;
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
        _ => None,
    }
}

/// This function launches the native eframe GUI application with specific starting paths.
pub fn run_gui(
    initial_folder_path: PathBuf,
    initial_result_file: Option<PathBuf>,
) -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 768.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Krets - Parquet Viewer",
        options,
        // Create the app instance with the provided paths
        Box::new(move |_cc| {
            Ok(Box::new(KretsApp::new(
                initial_folder_path,
                initial_result_file,
            )))
        }),
    )
}
