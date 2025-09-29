use std::sync::mpsc;

use antikythera::prelude::*;
use eframe::egui;

use crate::app::scripting::simulation::{LuaHook, LuaHookHandle};

const DEFAULT_HOOK_SCRIPT: &str = r#"-- Example Lua Hook Script
-- The global table `metrics` is available to store custom metrics

function on_integration_start(initial_state)
    -- Initialize any state or metrics here
end

function on_combat_start(state)
    -- Called at the start of each combat
end

function on_turn_start(state, actor_id, turn)
    -- Called at the start of each turn
end

function on_action_executed(state, action)
    -- Called after an action is executed
end

function on_turn_end(state, actor_id, turn)
    -- Called at the end of each turn
end

function on_combat_end(state)
    -- Called at the end of each combat
end

function on_integration_end()
    -- Finalize metrics here
end

"#;

pub struct SimulationApp {
    pub state: Option<State>,
    pub combats: usize,
    progress: f64,
    progress_rx: Option<mpsc::Receiver<f64>>,
    result_rx: Option<mpsc::Receiver<IntegrationResults>>,
    pub stats: Option<IntegrationResults>,
    pub hook_script: String,
    pub last_saved_hook_script: Option<String>,
    pub hook_handle: Option<LuaHookHandle>,
}

impl SimulationApp {
    pub fn new() -> Self {
        Self {
            state: None,
            combats: 1000,
            progress: 0.0,
            progress_rx: None,
            result_rx: None,
            stats: None,
            hook_handle: None,
            hook_script: String::from(DEFAULT_HOOK_SCRIPT),
            last_saved_hook_script: Some(String::from(DEFAULT_HOOK_SCRIPT)),
        }
    }

    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(last) = &self.last_saved_hook_script {
            &self.hook_script != last
        } else {
            true
        }
    }

    fn spawn_integrator(&mut self) {
        if let Some(state) = &self.state {
            let roller = Roller::new();
            let (hook, hook_handle) = LuaHook::new(self.hook_script.clone());
            self.hook_handle = Some(hook_handle);
            let mut integrator = Integrator::new(self.combats, roller, state.clone());
            integrator.add_hook(hook);
            let (progress_tx, progress_rx) = mpsc::channel();
            let (result_tx, result_rx) = mpsc::channel();
            let mut state_tree = StateTree::new(state.clone());
            integrator.start_time = chrono::Utc::now();
            std::thread::spawn({
                move || {
                    let total = integrator.min_combats as f64;
                    let mut last_reported = 0.0;
                    for hook in &mut integrator.hooks {
                        hook.on_integration_start(&integrator.initial_state);
                    }
                    while integrator.should_continue() {
                        integrator.run_combat(&mut state_tree).ok();
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
                        elapsed_time: elapsed,
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
            ui.label("Combats:");
            ui.add(
                egui::DragValue::new(&mut self.combats)
                    .range(1..=100000)
                    .speed(1),
            );
        });

        ui.separator();

        if ui.button("Start Simulation").clicked() && self.progress_rx.is_none() {
            log::info!("Starting simulation with {} combats", self.combats);
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
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label("Lua Hook Script:");
                if ui.button("Load").clicked() {
                    let should_continue = if self.has_unsaved_changes() {
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
                                self.hook_script = script;
                                self.last_saved_hook_script = None;
                                if let Some(handle) = &self.hook_handle
                                    && let Err(e) = handle.script_tx.send(self.hook_script.clone())
                                {
                                    log::error!("Failed to send script to hook: {}", e);
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to load script: {}", e);
                            }
                        }
                    }
                }
                if ui.button("Save").clicked()
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("Lua", &["lua"])
                        .set_title("Save Lua Script")
                        .set_file_name("hook.lua")
                        .save_file()
                {
                    match std::fs::write(&path, &self.hook_script) {
                        Ok(_) => {
                            self.last_saved_hook_script = Some(self.hook_script.clone());
                            log::info!("Script saved to {}", path.display());
                        }
                        Err(e) => {
                            log::error!("Failed to save script: {}", e);
                        }
                    }
                }
            });
            let text_editor = crate::app::lua_editor().show(ui, &mut self.hook_script);

            if text_editor.response.changed()
                && let Some(handle) = &self.hook_handle
                && let Err(e) = handle.script_tx.send(self.hook_script.clone())
            {
                log::error!("Failed to send script to hook: {}", e);
            }
        });
    }
}

impl Default for SimulationApp {
    fn default() -> Self {
        Self::new()
    }
}
