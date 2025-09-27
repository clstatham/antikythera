use antikythera::prelude::*;
use eframe::egui;

use crate::app::unsaved_changes_dialog;

#[derive(Default)]
struct StateEditorUiState {
    inventory_item_to_add: ItemId,
    name_editing: Option<(u32, String)>,
}

#[derive(Default)]
pub struct StateEditorApp {
    pub state: Option<State>,
    last_saved_state: Option<State>,
    ui_state: StateEditorUiState,
}

impl StateEditorApp {
    pub fn has_unsaved_changes(&self, state: &State) -> bool {
        if let Some(last_saved) = &self.last_saved_state {
            last_saved != state
        } else {
            true
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("State Editor");
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("New State").clicked() {
                let should_proceed = if let Some(state) = &self.state
                    && self.has_unsaved_changes(state)
                {
                    unsaved_changes_dialog()
                } else {
                    true
                };
                if should_proceed {
                    self.state = Some(State::new());
                    self.last_saved_state = self.state.clone();
                }
            }

            if ui.button("Load State").clicked() {
                let should_proceed = if let Some(state) = &self.state
                    && self.has_unsaved_changes(state)
                {
                    unsaved_changes_dialog()
                } else {
                    true
                };
                if should_proceed {
                    let dialog = rfd::FileDialog::new();
                    if let Some(path) = dialog.pick_file() {
                        let mut file = std::fs::File::open(&path).unwrap();
                        if let Ok(loaded_state) = serde_json::from_reader(&mut file) {
                            self.state = Some(loaded_state);
                            self.last_saved_state = self.state.clone();
                        } else {
                            log::error!("Failed to load state from file: {}", path.display());
                        }
                    }
                }
            }

            if ui.button("Save State").clicked()
                && let Some(state) = &self.state
            {
                let dialog = rfd::FileDialog::new();
                if let Some(path) = dialog.save_file() {
                    let mut file = std::fs::File::create(&path).unwrap();
                    if let Err(e) = serde_json::to_writer_pretty(&mut file, state) {
                        log::error!("Failed to save state to file: {}", e);
                    }
                    self.last_saved_state = Some(state.clone());
                }
            }
        });

        ui.separator();

        self.state_ui(ui);
    }

    fn actor_ui(
        ui: &mut egui::Ui,
        actor: ActorId,
        state: &mut State,
        ui_state: &mut StateEditorUiState,
    ) -> (bool, bool) {
        let Some(actor) = state.actors.get_mut(&actor) else {
            ui.label(format!("Actor ID {} not found in state.", actor.0));
            return (false, false);
        };

        let mut remove = false;
        let mut clone = false;

        egui::CollapsingHeader::new(format!("{}: {}", actor.id.0, actor.name))
            .id_salt(actor.id.0)
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Remove Actor").clicked() {
                        remove = true;
                    }
                    if ui.button("Clone Actor").clicked() {
                        clone = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Name:");

                    let name_field = if let Some((id, editing_name)) = &mut ui_state.name_editing
                        && id == &actor.id.0
                    {
                        ui.add(egui::TextEdit::singleline(editing_name).desired_width(200.0))
                    } else {
                        let mut name = actor.name.clone();
                        ui.add(egui::TextEdit::singleline(&mut name).desired_width(200.0))
                    };

                    if name_field.gained_focus() {
                        ui_state.name_editing = Some((actor.id.0, actor.name.clone()));
                    }
                    if name_field.lost_focus()
                        && let Some((id, _)) = &ui_state.name_editing
                        && id == &actor.id.0
                    {
                        actor.name = ui_state.name_editing.take().unwrap().1;
                    }
                });
                ui.horizontal(|ui| {
                    ui.label("NPC:");
                    ui.checkbox(&mut actor.npc, "");
                });
                ui.horizontal(|ui| {
                    ui.label("Group:");
                    ui.add(
                        egui::DragValue::new(&mut actor.group)
                            .speed(1)
                            .range(0..=100),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("HP:");
                    ui.add(
                        egui::DragValue::new(&mut actor.health)
                            .speed(0.5)
                            .range(0..=actor.max_health),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Max HP:");
                    ui.add(
                        egui::DragValue::new(&mut actor.max_health)
                            .speed(0.5)
                            .range(1..=1000),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("AC:");
                    ui.add(
                        egui::DragValue::new(&mut actor.armor_class)
                            .speed(0.5)
                            .range(1..=30),
                    );
                });

                egui::CollapsingHeader::new("Stats")
                    .default_open(false)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Strength:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Strength))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Dexterity:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Dexterity))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Constitution:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Constitution))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Intelligence:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Intelligence))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Wisdom:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Wisdom))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label("Charisma:");
                            ui.add(
                                egui::DragValue::new(actor.stats.get_mut(Stat::Charisma))
                                    .speed(1)
                                    .range(1..=20),
                            );
                        });
                    }); // end CollapsingHeader for Stats

                egui::CollapsingHeader::new("Saving Throws")
                    .default_open(false)
                    .show(ui, |ui| {
                        for save in SavingThrow::all() {
                            let mut proficient = actor.saving_throw_proficiencies.get(save);
                            let modifier = actor.saving_throw_modifier(save);
                            ui.horizontal(|ui| {
                                ui.label(format!("{:?}: {}", save, modifier));
                                ui.checkbox(&mut proficient, "Proficient");
                            });
                            actor.saving_throw_proficiencies.set(save, proficient);
                        }
                    }); // end CollapsingHeader for Saving Throws

                egui::CollapsingHeader::new("Skills")
                    .default_open(false)
                    .show(ui, |ui| {
                        for skill in Skill::all() {
                            let mut proficiency = actor.skill_proficiencies.get(skill);
                            ui.horizontal(|ui| {
                                ui.allocate_ui_with_layout(
                                    egui::Vec2::new(300.0, ui.available_height()),
                                    egui::Layout::left_to_right(egui::Align::Center),
                                    |ui| {
                                        ui.label(format!("{:?}:", skill));
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                egui::ComboBox::from_id_salt(format!(
                                                    "skill_{}",
                                                    skill as u32
                                                ))
                                                .selected_text(format!("{:?}", proficiency))
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(
                                                        &mut proficiency,
                                                        SkillProficiency::None,
                                                        "None",
                                                    );
                                                    ui.selectable_value(
                                                        &mut proficiency,
                                                        SkillProficiency::Proficient,
                                                        "Proficient",
                                                    );
                                                    ui.selectable_value(
                                                        &mut proficiency,
                                                        SkillProficiency::Expert,
                                                        "Expert",
                                                    );
                                                });
                                            },
                                        );
                                    },
                                );
                            });
                            actor.skill_proficiencies.set(skill, proficiency);
                        }
                    }); // end CollapsingHeader for Skills

                egui::CollapsingHeader::new("Inventory")
                    .default_open(false)
                    .show(ui, |ui| {
                        let num_items = actor.inventory.items.len();
                        let mut items_to_remove = Vec::new();
                        let mut items_to_add = Vec::new();

                        ui.horizontal(|ui| {
                            let item_id = &mut ui_state.inventory_item_to_add;
                            ui.add(
                                egui::DragValue::new(&mut item_id.0)
                                    .speed(0.5)
                                    .range(1..=state.next_item_id.saturating_sub(1)),
                            );
                            if ui.button("Add Item by ID").clicked() {
                                items_to_add.push((*item_id, 1));
                            }
                        });

                        ui.separator();

                        for (i, (item_id, quantity)) in actor.inventory.items.iter_mut().enumerate()
                        {
                            let Some(item) = state.items.get(item_id) else {
                                ui.label(format!(
                                    "Item ID {} not found in state items.",
                                    item_id.0
                                ));
                                continue;
                            };

                            if ui.button("Remove").clicked() {
                                items_to_remove.push(*item_id);
                                continue;
                            }

                            ui.horizontal(|ui| {
                                ui.label(format!("Item ID: {}", item_id.0));
                            });
                            ui.horizontal(|ui| {
                                ui.label("Item Name:");
                                ui.label(&item.name);
                            });

                            ui.horizontal(|ui| {
                                ui.label("Quantity:");
                                ui.add(egui::DragValue::new(quantity).speed(1).range(0..=100));
                            });

                            if i < num_items - 1 {
                                ui.separator();
                            }
                        }

                        for item_id in items_to_remove {
                            actor.inventory.remove_item(item_id, 1000);
                        }

                        for (item_id, quantity) in items_to_add {
                            actor.inventory.add_item(item_id, quantity);
                        }
                    }); // end CollapsingHeader for Inventory
            }); // end CollapsingHeader for Actor

        (remove, clone)
    }

    // NOTE: These two functions NO LONGER create their own ScrollAreas.
    // The scroll is now provided by the pane that contains them, so they
    // can expand naturally to the full height of the strip cell.
    fn actors_list_ui(ui: &mut egui::Ui, state: &mut State, ui_state: &mut StateEditorUiState) {
        egui::CollapsingHeader::new("Actors")
            .default_open(false)
            .show(ui, |ui| {
                if ui.button("Add Actor").clicked() {
                    let new_actor = ActorBuilder::new("New Actor").build();
                    state.add_actor(new_actor);
                }

                let actors: Vec<ActorId> = state.actors.keys().cloned().collect();
                for actor_id in actors {
                    let (remove, clone) = Self::actor_ui(ui, actor_id, state, ui_state);
                    if remove {
                        state.actors.remove(&actor_id);
                    }
                    if clone && let Some(actor) = state.actors.get(&actor_id) {
                        let mut cloned_actor = actor.clone();
                        let new_id = state.next_actor_id;
                        cloned_actor.id = ActorId(new_id);
                        state.add_actor(cloned_actor);
                    }
                }
            }); // end CollapsingHeader for Actors
    }

    fn item_ui(
        ui: &mut egui::Ui,
        item_id: ItemId,
        state: &mut State,
        ui_state: &mut StateEditorUiState,
    ) {
        let Some(item) = state.items.get_mut(&item_id) else {
            ui.label(format!("Item ID {} not found in state.", item_id.0));
            return;
        };

        egui::CollapsingHeader::new(format!("{}: {}", item.id.0, item.name))
            .id_salt(item.id.0)
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Name:");

                    let name_field = if let Some((id, editing_name)) = &mut ui_state.name_editing
                        && id == &item.id.0
                    {
                        ui.add(egui::TextEdit::singleline(editing_name).desired_width(200.0))
                    } else {
                        let mut name = item.name.clone();
                        ui.add(egui::TextEdit::singleline(&mut name).desired_width(200.0))
                    };

                    if name_field.gained_focus() {
                        ui_state.name_editing = Some((item.id.0, item.name.clone()));
                    }
                    if name_field.lost_focus()
                        && let Some((id, _)) = &ui_state.name_editing
                        && id == &item.id.0
                    {
                        item.name = ui_state.name_editing.take().unwrap().1;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Type:");
                    ui.label(format!("{:?}", item.item_type()));
                });

                match &mut item.inner {
                    ItemInner::Weapon(weapon) => {
                        egui::CollapsingHeader::new("Weapon Details")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Weapon Type:");
                                    egui::ComboBox::from_id_salt("weapon_type")
                                        .selected_text(format!("{:?}", weapon.weapon_type))
                                        .show_ui(ui, |ui| {
                                            for wt in WeaponType::all() {
                                                ui.selectable_value(
                                                    &mut weapon.weapon_type,
                                                    *wt,
                                                    format!("{:?}", wt),
                                                );
                                            }
                                        });
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Damage:");
                                    let mut formula = String::new();
                                    weapon.damage.pretty_print(&mut formula).unwrap();
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut formula)
                                                .desired_width(100.0),
                                        )
                                        .changed()
                                        && let Ok(parsed) =
                                            antikythera::roll_parser::parse_roll(&formula)
                                    {
                                        weapon.damage = parsed;
                                    }
                                });

                                ui.horizontal(|ui| {
                                    ui.label("Critical Damage:");
                                    let mut formula = String::new();
                                    let critical_damage =
                                        weapon.critical_damage.as_ref().unwrap_or(&weapon.damage);
                                    critical_damage.pretty_print(&mut formula).unwrap();
                                    if ui
                                        .add(
                                            egui::TextEdit::singleline(&mut formula)
                                                .desired_width(100.0),
                                        )
                                        .changed()
                                        && let Ok(parsed) =
                                            antikythera::roll_parser::parse_roll(&formula)
                                    {
                                        weapon.critical_damage = Some(parsed);
                                    }
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Attack Bonus:");
                                    ui.add(
                                        egui::DragValue::new(&mut weapon.attack_bonus)
                                            .speed(1)
                                            .range(-10..=10),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Range:");
                                    // melee checkbox
                                    let mut is_melee = weapon.range.is_none();
                                    if ui.checkbox(&mut is_melee, "Melee").changed() {
                                        if is_melee {
                                            weapon.range = None;
                                        } else {
                                            weapon.range = Some(20);
                                        }
                                    }
                                    if let Some(range) = &mut weapon.range {
                                        ui.add(
                                            egui::DragValue::new(range).speed(5).range(0..=1000),
                                        );
                                    }
                                });
                            }); // end CollapsingHeader for Weapon Details
                    }
                    ItemInner::Armor(armor) => {
                        egui::CollapsingHeader::new("Armor Details")
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("AC Bonus:");
                                    ui.add(
                                        egui::DragValue::new(&mut armor.ac_bonus)
                                            .speed(1)
                                            .range(1..=30),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    ui.label("Stealth Disadvantage:");
                                    ui.checkbox(&mut armor.stealth_disadvantage, "");
                                });
                            }); // end CollapsingHeader for Armor Details
                    }
                    _ => {}
                }
            }); // end CollapsingHeader for item
    }

    fn items_list_ui(ui: &mut egui::Ui, state: &mut State, _ui_state: &mut StateEditorUiState) {
        egui::CollapsingHeader::new("Items")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Add Weapon").clicked() {
                        let weapon = WeaponBuilder::new(WeaponType::Longsword)
                            .attack_bonus(0)
                            .damage("1d8")
                            .build();
                        state.add_item("New Weapon", ItemInner::Weapon(weapon));
                    }
                    if ui.button("Add Armor").clicked() {
                        let armor = Armor {
                            ac_bonus: 1,
                            stealth_disadvantage: false,
                        };
                        state.add_item("New Armor", ItemInner::Armor(armor));
                    }
                });

                let items: Vec<ItemId> = state.items.keys().cloned().collect();
                for item_id in items {
                    Self::item_ui(ui, item_id, state, _ui_state);
                }
            }); // end CollapsingHeader for Items
    }

    fn state_ui(&mut self, ui: &mut egui::Ui) {
        let Some(state) = &mut self.state else {
            ui.label("No state loaded. Create or load a state to begin editing.");
            return;
        };
        ui.label(format!("Actors: {}", state.actors.len()));
        ui.label(format!("Items: {}", state.items.len()));
        ui.separator();

        // Fill all remaining area below the stats/separator with a 2-col strip.
        egui::CentralPanel::default().show_inside(ui, |ui| {
            egui_extras::StripBuilder::new(ui)
                .size(egui_extras::Size::remainder())
                .size(egui_extras::Size::remainder())
                .horizontal(|mut strip| {
                    // Left pane (Actors)
                    strip.cell(|ui| {
                        let avail = ui.available_size();
                        ui.allocate_ui_with_layout(
                            avail,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(
                                    ui,
                                    |ui| {
                                        Self::actors_list_ui(ui, state, &mut self.ui_state);
                                    },
                                );
                            },
                        );
                    });

                    // Right pane (Items)
                    strip.cell(|ui| {
                        let avail = ui.available_size();
                        ui.allocate_ui_with_layout(
                            avail,
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                egui::ScrollArea::vertical().auto_shrink([false; 2]).show(
                                    ui,
                                    |ui| {
                                        Self::items_list_ui(ui, state, &mut self.ui_state);
                                    },
                                );
                            },
                        );
                    });
                });
        });
    }
}
