use std::sync::mpsc;

use antikythera::prelude::*;
use eframe::egui;

use crate::app::scripting::{LuaHook, LuaHookHandle};

const DEFAULT_HOOK_SCRIPT: &str = r#"-- Example Lua Hook Script
function on_integration_start(initial_state)
    metrics["combats_started"] = 0
    metrics["actions_executed"] = 0
end

function on_combat_start(state)
    print("Combat started with " .. #state.actors .. " actors.")
    metrics["combats_started"] = metrics["combats_started"] + 1
end

function on_turn_start(state, actor_id, turn)
    print("Turn " .. turn .. " started for actor " .. actor_id)
end

function on_action_executed(state, action)
    print("Action executed: " .. action.action_type)
    metrics["actions_executed"] = metrics["actions_executed"] + 1
end

function on_turn_end(state, actor_id, turn)
    print("Turn " .. turn .. " ended for actor " .. actor_id)
end

function on_combat_end(state)
    print("Combat ended.")
end

function on_integration_end()
    print("Integration ended.")
    for k, v in pairs(metrics) do
        print(k .. ": " .. v)
    end
end

"#;

pub struct SimulationApp {
    pub state: Option<State>,
    pub min_combats: usize,
    progress: f64,
    progress_rx: Option<mpsc::Receiver<f64>>,
    result_rx: Option<mpsc::Receiver<IntegrationResults>>,
    pub stats: Option<IntegrationResults>,
    pub hook_script: String,
    pub hook_handle: Option<LuaHookHandle>,
}

impl SimulationApp {
    pub fn new() -> Self {
        Self {
            state: None,
            min_combats: 1000,
            progress: 0.0,
            progress_rx: None,
            result_rx: None,
            stats: None,
            hook_handle: None,
            hook_script: String::from(DEFAULT_HOOK_SCRIPT),
        }
    }

    fn spawn_integrator(&mut self) {
        if let Some(state) = &self.state {
            let roller = Roller::new();
            let (hook, hook_handle) = LuaHook::new(self.hook_script.clone());
            self.hook_handle = Some(hook_handle);
            let mut integrator = Integrator::new(self.min_combats, roller, state.clone());
            integrator.add_hook(hook);
            let (progress_tx, progress_rx) = mpsc::channel();
            let (result_tx, result_rx) = mpsc::channel();
            let mut state_tree = StateTree::new(state.clone());
            integrator.start_time = chrono::Utc::now();
            std::thread::spawn({
                let mut roller = integrator.roller.fork();
                move || {
                    let total = integrator.min_combats as f64;
                    let mut last_reported = 0.0;
                    for hook in &mut integrator.hooks {
                        hook.on_integration_start(&integrator.initial_state);
                    }
                    while integrator.should_continue() {
                        integrator.run_combat(roller.fork(), &mut state_tree).ok();
                        let completed = integrator.combats_run() as f64;
                        let progress = completed / total;
                        if (progress - last_reported) >= 0.01 || progress == 1.0 {
                            last_reported = progress;
                            let _ = progress_tx.send(progress);
                        }
                    }

                    let elapsed = integrator.elapsed_time();

                    for hook in &mut integrator.hooks {
                        hook.on_integration_end();
                    }

                    let mut hook_metrics = Vec::new();
                    for hook in &integrator.hooks {
                        hook_metrics.extend(hook.metrics());
                    }

                    let results = IntegrationResults {
                        state_tree: state_tree.clone(),
                        combats_run: integrator.combats_run(),
                        elapsed,
                        hook_metrics,
                    };

                    let _ = result_tx.send(results);
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
                    log::info!("Simulation completed.");
                    self.stats = Some(state_tree);
                    self.progress_rx = None;
                    self.result_rx = None;
                } else {
                    ui.label("Simulation running...");
                }
            }
        }

        if let Some(results) = &self.stats {
            ui.separator();
            ui.label(format!(
                "Simulation Results: {} states, {} transitions",
                results.state_tree.graph.node_count(),
                results.state_tree.graph.edge_count()
            ));

            if ui.button("Save Results").clicked()
                && let Some(path) = rfd::FileDialog::new()
                    .add_filter("JSON", &["json"])
                    .set_file_name("antikythera-results.json")
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

        ui.separator();

        // Display our hooks script
        ui.label("Lua Hook Script:");
        let script_changed = ui
            .add(egui::TextEdit::multiline(&mut self.hook_script).code_editor())
            .changed();
        if script_changed
            && let Some(handle) = &self.hook_handle
            && let Err(e) = handle.script_tx.send(self.hook_script.clone())
        {
            log::error!("Failed to send script to hook: {}", e);
        }
    }
}

impl Default for SimulationApp {
    fn default() -> Self {
        Self::new()
    }
}
