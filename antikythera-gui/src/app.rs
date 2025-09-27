use antikythera::prelude::*;
use eframe::egui;
use serde::{Deserialize, Serialize};

pub mod analysis;
pub mod simulation;
pub mod state_editor;

#[derive(Debug, Default, PartialEq)]
pub enum AppMode {
    #[default]
    Home,
    StateEditor,
    Simulation,
    Analysis,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Statistics {
    pub initial_state: State,
    pub total_combats: usize,
    pub state_tree: StateTree,
    pub state_tree_stats: StateTreeStats,
}

#[derive(Default)]
pub struct App {
    pub mode: AppMode,
    pub state: Option<State>,
    pub stats: Option<Statistics>,
    pub state_editor_app: state_editor::StateEditorApp,
    pub simulation_app: simulation::SimulationApp,
    pub analysis_app: analysis::AnalysisApp,
}

impl App {
    pub fn ui(&mut self, ctx: &egui::Context) {
        ctx.request_repaint();

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.mode == AppMode::Home, "Home")
                    .clicked()
                {
                    self.mode_transition(AppMode::Home);
                }
                if ui
                    .selectable_label(self.mode == AppMode::StateEditor, "State Editor")
                    .clicked()
                {
                    self.mode_transition(AppMode::StateEditor);
                }
                if ui
                    .selectable_label(self.mode == AppMode::Simulation, "Simulation")
                    .clicked()
                {
                    self.mode_transition(AppMode::Simulation);
                }
                if ui
                    .selectable_label(self.mode == AppMode::Analysis, "Analysis")
                    .clicked()
                {
                    self.mode_transition(AppMode::Analysis);
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.mode {
            AppMode::Home => {
                ui.label("Welcome to the Antikythera Mechanism. What frightening answers thou mayest find here.");
                ui.label("Use the tabs above to navigate between different tools.");
            }
            AppMode::StateEditor => {
                self.state_editor_app.ui(ui);
            }
            AppMode::Simulation => {
                self.simulation_app.ui(ui);
            }
            AppMode::Analysis => {
                self.analysis_app.ui(ui);
            }
        });
    }

    fn mode_transition(&mut self, new_mode: AppMode) {
        if self.mode == new_mode {
            return;
        }

        let state = match self.mode {
            AppMode::Home => self.state.take(),
            AppMode::StateEditor => self.state_editor_app.state.take(),
            AppMode::Simulation => self.simulation_app.state.take(),
            AppMode::Analysis => self.state.take(),
        };

        if let Some(state) = state {
            match new_mode {
                AppMode::StateEditor => self.state_editor_app.state = Some(state),
                AppMode::Simulation => self.simulation_app.state = Some(state),
                AppMode::Home => self.state = Some(state),
                AppMode::Analysis => self.state = Some(state),
            }
        }

        let stats = match self.mode {
            AppMode::Simulation => self.simulation_app.stats.take(),
            AppMode::Analysis => self.analysis_app.stats.take(),
            _ => self.stats.take(),
        };

        if let Some(stats) = stats {
            match new_mode {
                AppMode::Analysis => self.analysis_app.stats = Some(stats),
                AppMode::Simulation => self.simulation_app.stats = Some(stats),
                _ => self.stats = Some(stats),
            }
        }

        self.mode = new_mode;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);

        if ctx.input(|r| r.viewport().close_requested())
            && self.state_editor_app.has_unsaved_changes()
        {
            let should_proceed = unsaved_changes_dialog();
            if should_proceed {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            }
        }
    }
}

pub fn unsaved_changes_dialog() -> bool {
    let confirm = rfd::MessageDialog::new()
        .set_title("Unsaved Changes")
        .set_description("Discard unsaved changes?")
        .set_buttons(rfd::MessageButtons::YesNo)
        .set_level(rfd::MessageLevel::Warning)
        .show();
    confirm == rfd::MessageDialogResult::Yes
}
