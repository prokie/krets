use arrow::record_batch::RecordBatch;
use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui_plot::{Legend, Line, Plot, PlotPoints};
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
                    // We will have 3 columns: Column Name, Min, Max
                    .columns(Column::auto(), 3)
                    .sense(egui::Sense::click());

                table
                    .header(20.0, |mut header| {
                        header.col(|ui| {
                            ui.strong("Column");
                        });
                        header.col(|ui| {
                            ui.strong("Min");
                        });
                        header.col(|ui| {
                            ui.strong("Max");
                        });
                    })
                    .body(|mut body| {
                        // Iterate over the *columns* in the batch.
                        // Each column will be a *row* in our new table.
                        for (col_index, array) in data.batch.columns().iter().enumerate() {
                            let column_name = &data.headers[col_index];

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

                                let response = row.response();
                                if response.clicked() {
                                    // This is the logic from the example's `toggle_row_selection`
                                    if self.selection.contains(&col_index) {
                                        self.selection.remove(&col_index);
                                    } else {
                                        // Use this for multi-selection:
                                        self.selection.insert(col_index);

                                        // Or use this for single-selection:
                                        // self.selection.clear();
                                        // self.selection.insert(col_index);
                                    }
                                }
                            });
                        }
                    });
            } else {
                ui.label("Select a .parquet file from the explorer to view its data.");
            }

            // You can keep the plot or remove it. For this example, I'll keep it.
            ui.separator();
            ui.heading("Plot Viewer");
            let my_plot = Plot::new("My Plot").legend(Legend::default());
            let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
            my_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new("curve", PlotPoints::from(graph)));
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
