use eframe::egui;

use crate::app::Statistics;

#[derive(Default)]
pub struct AnalysisApp {
    pub stats: Option<Statistics>,
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
                .and_then(|data| serde_json::from_str::<Statistics>(&data).map_err(|e| e.into()))
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

        ui.separator();

        if let Some(stats) = &self.stats {
            ui.label(format!(
                "Results loaded: {} states, {} transitions, {} combats",
                stats.state_tree.graph.node_count(),
                stats.state_tree.graph.edge_count(),
                stats.total_combats,
            ));
        }
    }
}
