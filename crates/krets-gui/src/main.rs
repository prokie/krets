use std::{fs, path::PathBuf};

use eframe::egui;
use egui_plot::{Legend, Line, Plot, PlotPoints};
use egui_table::Table;

/// Represents an entry in the directory listing.
struct DirectoryEntry {
    path: PathBuf,
    is_directory: bool,
}

struct KretsApp {
    current_path: PathBuf,
    entries: Vec<DirectoryEntry>,
    error_message: Option<String>,
}

impl Default for KretsApp {
    fn default() -> Self {
        // Start in the current working directory.
        let mut app = Self {
            current_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
            entries: Vec::new(),
            error_message: None,
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

            // --- Path and Controls Section ---
            ui.horizontal(|ui| {
                if ui.button("⬆ Up").clicked() {
                    if let Some(parent) = self.current_path.parent() {
                        // Record the intent to navigate up.
                        path_to_navigate = Some(parent.to_path_buf());
                    }
                }
                // Display the current path as a text label.
                ui.label(format!("Current Path: {}", self.current_path.display()));
            });

            ui.separator();

            // --- Directory Listing Section ---
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            } else {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for entry in &self.entries {
                        let icon = if entry.is_directory { "📁" } else { "📄" };
                        let file_name =
                            entry.path.file_name().unwrap_or_default().to_string_lossy();

                        // Directories are clickable buttons that change the path.
                        if entry.is_directory {
                            if ui.button(format!("{icon} {file_name}")).clicked() {
                                // Record the intent to navigate into this directory.
                                path_to_navigate = Some(entry.path.clone());
                            }
                        } else {
                            // Files are just labels for now.
                            ui.label(format!("{icon} {file_name}"));
                        }
                    }
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Plot Viewer");
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }

            let my_plot = Plot::new("My Plot").legend(Legend::default());
            let my_table = Table::new();

            my_table.columns(vec![
                egui_table::Column::new(0.0),
                egui_table::Column::new(1.1),
                egui_table::Column::new(2.2),
            ]);

            my_table.headers(vec!["Column 1".to_string().into()]);

            // let's create a dummy line in the plot
            let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
            my_plot.show(ui, |plot_ui| {
                plot_ui.line(Line::new("curve", PlotPoints::from(graph)));
            })
        });

        // --- Apply State Changes ---
        // If a navigation action was recorded, apply it now.
        if let Some(new_path) = path_to_navigate {
            self.current_path = new_path;
            self.refresh_entries();
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
}

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Native file dialogs and drag-and-drop files",
        options,
        Box::new(|_cc| Ok(Box::<KretsApp>::default())),
    )
}
