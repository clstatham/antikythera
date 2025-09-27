use std::sync::mpsc;

use antikythera::prelude::*;
use eframe::egui;

use crate::app::Statistics;

#[derive(Default)]
pub struct SimulationApp {
    pub state: Option<State>,
    pub min_combats: usize,
    progress: f64,
    progress_rx: Option<mpsc::Receiver<f64>>,
    result_rx: Option<mpsc::Receiver<StateTree>>,
    pub stats: Option<Statistics>,
}

impl SimulationApp {
    fn spawn_integrator(&mut self) {
        if let Some(state) = &self.state {
            let roller = Roller::new();
            let mut integrator = Integrator::new(self.min_combats, roller, state.clone());
            let (progress_tx, progress_rx) = mpsc::channel();
            let (result_tx, result_rx) = mpsc::channel();
            let mut state_tree = StateTree::new(state.clone());
            integrator.start_time = chrono::Utc::now();
            std::thread::spawn({
                let mut roller = integrator.roller.fork();
                move || {
                    let total = integrator.min_combats as f64;
                    let mut last_reported = 0.0;
                    while integrator.should_continue() {
                        integrator.run_combat(roller.fork(), &mut state_tree).ok();
                        let completed = integrator.combats_run() as f64;
                        let progress = completed / total;
                        if (progress - last_reported) >= 0.01 || progress == 1.0 {
                            last_reported = progress;
                            let _ = progress_tx.send(progress);
                        }
                    }
                    let _ = result_tx.send(state_tree);
                }
            });
            self.progress_rx = Some(progress_rx);
            self.result_rx = Some(result_rx);
        } else {
            self.progress_rx = None;
            self.result_rx = None;
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("Simulation");
        });

        ui.separator();

        if self.state.is_none() {
            ui.label("Please load or create a state in the State Editor first.");
            return;
        }

        ui.horizontal(|ui| {
            ui.label("Minimum Combats:");
            ui.add(
                egui::DragValue::new(&mut self.min_combats)
                    .range(1..=100000)
                    .speed(1),
            );
        });

        ui.separator();

        if ui.button("Start Simulation").clicked() && self.progress_rx.is_none() {
            log::info!(
                "Starting simulation with {} minimum combats",
                self.min_combats
            );
            self.spawn_integrator();
        }

        if self.progress_rx.is_some() {
            ui.label("Simulation started...");

            // show a progress bar
            if let Some(progress_rx) = &self.progress_rx
                && let Ok(progress) = progress_rx.try_recv()
            {
                self.progress = progress;
            }

            ui.add(egui::ProgressBar::new(self.progress as f32).show_percentage());

            // check for results
            if let Some(result_rx) = &self.result_rx {
                if let Ok(state_tree) = result_rx.try_recv() {
                    log::info!("Simulation completed, calculating statistics...");
                    let stats = Statistics {
                        initial_state: self.state.clone().unwrap_or_default(),
                        total_combats: self.min_combats,
                        state_tree: state_tree.clone(),
                        state_tree_stats: state_tree.compute_statistics(),
                    };
                    log::info!("Statistics calculated.");
                    self.stats = Some(stats);
                    self.progress_rx = None;
                    self.result_rx = None;
                } else {
                    ui.label("Simulation running...");
                }
            }
        }

        if let Some(results) = &self.stats {
            ui.label(format!(
                "Simulation completed with {} states and {} transitions explored.",
                results.state_tree.graph.node_count(),
                results.state_tree.graph.edge_count()
            ));

            if ui.button("Save Results").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("antikythera-statistics.json")
                    .save_file()
            {
                let file = match std::fs::File::create(&path) {
                    Ok(file) => file,
                    Err(e) => {
                        log::error!("Failed to create file {}: {}", path.display(), e);
                        return;
                    }
                };
                let writer = std::io::BufWriter::new(file);
                match serde_json::to_writer(writer, &results) {
                    Ok(_) => {
                        log::info!("Results saved to {}", path.display());
                    }
                    Err(e) => {
                        log::error!("Failed to write results to {}: {}", path.display(), e);
                    }
                }
            }

            if ui.button("Clear Results").clicked() {
                log::info!("Clearing results from memory.");
                self.stats = None;
            }
        }
    }
}
