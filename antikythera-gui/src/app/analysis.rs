use antikythera::prelude::*;
use eframe::egui;

use crate::app::scripting::analysis::AnalysisScriptInterface;

pub struct Metric {
    pub query_name: String,
    pub result: String,
}

#[derive(Default)]
pub struct AnalysisApp {
    pub stats: Option<IntegrationResults>,
    metrics: Vec<Metric>,
    script_interface: AnalysisScriptInterface,
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
            match std::fs::read_to_string(&path).and_then(|data| {
                serde_json::from_str::<IntegrationResults>(&data).map_err(|e| e.into())
            }) {
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
                stats.state_tree.node_count()
            ));

            ui.separator();

            ui.horizontal(|ui| {
                ui.label("Analysis Script:");
                if ui.button("Load").clicked() {
                    let should_continue = if self.script_interface.has_unsaved_changes() {
                        crate::app::unsaved_changes_dialog()
                    } else {
                        true
                    };

                    if should_continue
                        && let Some(path) = rfd::FileDialog::new()
                            .add_filter("Lua", &["lua"])
                            .set_title("Select Lua Script")
                            .pick_file()
                    {
                        match std::fs::read_to_string(&path) {
                            Ok(script) => {
                                self.script_interface.query = script;
                                self.script_interface.last_saved_query =
                                    Some(self.script_interface.query.clone());
                                self.script_interface.script_error = None;
                            }
                            Err(e) => {
                                self.script_interface.script_error =
                                    Some(format!("Failed to load script: {}", e));
                            }
                        }
                    }
                }
                if ui.button("Save").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Lua", &["lua"])
                        .set_title("Save Lua Script")
                        .set_file_name("analysis.lua")
                        .save_file()
                {
                    match std::fs::write(&path, &self.script_interface.query) {
                        Ok(_) => {
                            self.script_interface.last_saved_query =
                                Some(self.script_interface.query.clone());
                            self.script_interface.script_error = None;
                        }
                        Err(e) => {
                            self.script_interface.script_error =
                                Some(format!("Failed to save script: {}", e));
                        }
                    }
                }
            });

            let text_editor_output =
                crate::app::lua_editor().show(ui, &mut self.script_interface.query);
            if text_editor_output.response.changed() {
                // Clear previous error if any
                self.script_interface.script_error = None;
            }

            // Run query on Ctrl+Enter
            if text_editor_output.response.has_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter) && i.modifiers.ctrl)
            {
                text_editor_output.response.request_focus(); // keep focus after Ctrl+Enter
                if let Err(e) = self
                    .script_interface
                    .run_outcome_probability_query(&stats.state_tree)
                {
                    self.script_interface.script_error =
                        Some(format!("Error running query: {}", e));
                }
            }

            ui.checkbox(
                &mut self.script_interface.externals_only,
                "Run on terminal states only",
            );

            if ui.button("Run Query").clicked()
                && let Some(results) = self.stats.as_ref()
            {
                match self
                    .script_interface
                    .run_outcome_probability_query(&results.state_tree)
                {
                    Ok(probability) => {
                        self.metrics.push(Metric {
                            query_name: if self.script_interface.externals_only {
                                format!(
                                    "Terminal State Probability of:\n{}",
                                    self.script_interface.query
                                )
                            } else {
                                format!("State Probability of:\n{}", self.script_interface.query)
                            },
                            result: format!("{}%", probability * 100.0),
                        });

                        self.script_interface.script_error = None;
                    }
                    Err(e) => {
                        self.script_interface.script_error =
                            Some(format!("Error running query: {}", e));
                    }
                }
            }

            if let Some(error) = &self.script_interface.script_error {
                ui.colored_label(egui::Color32::RED, error);
            }

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
                        for (name, value) in &stats.hook_metrics {
                            ui.label(name);
                            ui.label(format!("{}", value));
                            ui.end_row();
                        }
                        for (name, value) in &self.script_interface.metrics {
                            ui.label(name);
                            ui.label(format!("{}", value));
                            ui.end_row();
                        }
                    });
            });
        }
    }
}
