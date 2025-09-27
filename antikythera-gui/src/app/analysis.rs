use antikythera::prelude::*;
use eframe::egui;

use crate::app::analysis::scripting::ScriptInterface;

pub mod scripting;

pub struct Metric {
    pub query_name: String,
    pub result: String,
}

#[derive(Default)]
pub struct AnalysisApp {
    pub stats: Option<StateTree>,
    metrics: Vec<Metric>,
    script_interface: ScriptInterface,
}

impl AnalysisApp {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Analysis");
        });

        ui.separator();

        if ui.button("Load Results").clicked()
            && let Some(path) = rfd::FileDialog::new()
                .add_filter("JSON", &["json"])
                .set_title("Select Results File")
                .pick_file()
        {
            match std::fs::read_to_string(&path)
                .and_then(|data| serde_json::from_str::<StateTree>(&data).map_err(|e| e.into()))
            {
                Ok(stats) => {
                    self.stats = Some(stats);
                }
                Err(e) => {
                    eprintln!("Failed to load results: {}", e);
                }
            }
        }

        if self.stats.is_some() && ui.button("Clear Results").clicked() {
            self.stats = None;
        }

        if let Some(stats) = &self.stats {
            ui.label(format!(
                "Loaded state tree with {} nodes",
                stats.graph.node_count()
            ));

            ui.separator();
            self.script_interface.ui(ui, &self.stats, &mut self.metrics);

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                egui::Grid::new("metrics_grid")
                    .striped(true)
                    .min_col_width(200.0)
                    .show(ui, |ui| {
                        ui.heading("Metric");
                        ui.heading("Result");
                        ui.end_row();
                        for metric in &self.metrics {
                            ui.label(&metric.query_name);
                            ui.label(&metric.result);
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
