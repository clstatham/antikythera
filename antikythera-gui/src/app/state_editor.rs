use antikythera::prelude::*;
use eframe::egui;

#[derive(Default)]
pub struct StateEditorApp {
    pub state: Option<State>,
}

impl StateEditorApp {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("State Editor");
        });

        ui.separator();

        ui.horizontal(|ui| {
            if ui.button("New State").clicked() {
                self.state = Some(State::new());
            }

            if ui.button("Load State").clicked() {
                let dialog = rfd::FileDialog::new();
                if let Some(path) = dialog.pick_file() {
                    let mut file = std::fs::File::open(&path).unwrap();
                    if let Ok(loaded_state) = serde_json::from_reader(&mut file) {
                        self.state = Some(loaded_state);
                    } else {
                        log::error!("Failed to load state from file: {}", path.display());
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
                }
            }
        });

        ui.separator();

        self.state_ui(ui);
    }

    fn state_ui(&mut self, ui: &mut egui::Ui) {
        let Some(state) = &mut self.state else {
            ui.label("No state loaded. Create or load a state to begin editing.");
            return;
        };
        ui.label(format!("Actors: {}", state.actors.len()));
        ui.label(format!("Allied Groups: {}", state.allied_groups.len()));
        ui.label(format!("Items: {}", state.items.len()));
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
        egui::CollapsingHeader::new("Actors")
                        .default_open(false)
                        .show(ui, |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (_actor_id, actor) in state.actors.iter_mut() {
                                    egui::CollapsingHeader::new(format!("{}", actor.id.0))
                                        .default_open(false)
                                        .show(ui, |ui| {
                                            ui.horizontal(|ui| {
                                                ui.label("Name:");
                                                ui.add(
                                                    egui::TextEdit::singleline(&mut actor.name)
                                                        .desired_width(200.0),
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
                                                            egui::DragValue::new(
                                                                actor.stats.get_mut(Stat::Strength),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Dexterity:");
                                                        ui.add(
                                                            egui::DragValue::new(
                                                                actor
                                                                    .stats
                                                                    .get_mut(Stat::Dexterity),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Constitution:");
                                                        ui.add(
                                                            egui::DragValue::new(
                                                                actor
                                                                    .stats
                                                                    .get_mut(Stat::Constitution),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Intelligence:");
                                                        ui.add(
                                                            egui::DragValue::new(
                                                                actor
                                                                    .stats
                                                                    .get_mut(Stat::Intelligence),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Wisdom:");
                                                        ui.add(
                                                            egui::DragValue::new(
                                                                actor.stats.get_mut(Stat::Wisdom),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                    ui.horizontal(|ui| {
                                                        ui.label("Charisma:");
                                                        ui.add(
                                                            egui::DragValue::new(
                                                                actor.stats.get_mut(Stat::Charisma),
                                                            )
                                                            .speed(1)
                                                            .range(1..=20),
                                                        );
                                                    });
                                                });

                                            egui::CollapsingHeader::new("Saving Throws")
                                                .default_open(false)
                                                .show(ui, |ui| {
                                                    for save in SavingThrow::all() {
                                                        let mut proficient = actor
                                                            .saving_throw_proficiencies
                                                            .get(save);
                                                        let modifier = actor.saving_throw_modifier(save);
                                                        ui.horizontal(|ui| {
                                                            ui.label(format!("{:?}: {}", save, modifier));
                                                            ui.checkbox(
                                                                &mut proficient,
                                                                "Proficient",
                                                            );
                                                        });
                                                        actor
                                                            .saving_throw_proficiencies
                                                            .set(save, proficient);
                                                    }
                                                });

                                            egui::CollapsingHeader::new("Skills")
                                                .default_open(false)
                                                .show(ui, |ui| {
                                                    for skill in Skill::all() {
                                                        let mut proficiency =
                                                            actor.skill_proficiencies.get(skill);
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
                                                                            .selected_text(format!(
                                                                                "{:?}",
                                                                                proficiency
                                                                            ))
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
                                                        actor
                                                            .skill_proficiencies
                                                            .set(skill, proficiency);
                                                    }
                                                });

                                            egui::CollapsingHeader::new("Inventory")
                                                .default_open(false)
                                                .show(ui, |ui| {
                                                    let num_items = actor.inventory.items.len();
                                                    for (i, (item_id, entry)) in
                                                        actor.inventory.items.iter_mut().enumerate()
                                                    {
                                                        ui.horizontal(|ui| {
                                                            ui.label("Item ID:");
                                                            ui.label(format!("{}", item_id.0));
                                                        });
                                                        ui.horizontal(|ui| {
                                                            ui.label("Item Name:");
                                                            ui.label(&entry.item.name);
                                                        });

                                                        ui.horizontal(|ui| {
                                                            ui.label("Quantity:");
                                                            ui.add(
                                                                egui::DragValue::new(
                                                                    &mut entry.quantity,
                                                                )
                                                                .speed(1)
                                                                .range(0..=100),
                                                            );
                                                        });

                                                        if i < num_items - 1 {
                                                            ui.separator();
                                                        }
                                                    }
                                                });
                                        });
                                }
                            });
                        });

        egui::CollapsingHeader::new("Items")
            .default_open(false)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for (_item_id, item) in state.items.iter_mut() {
                        egui::CollapsingHeader::new(format!("{}", item.id.0))
                            .default_open(false)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label("Name:");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut item.name)
                                            .desired_width(200.0),
                                    );
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
                                                    ui.label(format!("{:?}", weapon.weapon_type));
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Damage:");
                                                    let mut formula = String::new();
                                                    weapon
                                                        .damage
                                                        .pretty_print(&mut formula)
                                                        .unwrap();
                                                    if ui
                                                        .add(
                                                            egui::TextEdit::singleline(
                                                                &mut formula,
                                                            )
                                                            .desired_width(100.0),
                                                        )
                                                        .changed()
                                                        && let Ok(parsed) =
                                                            antikythera::roll_parser::parse_roll(
                                                                &formula,
                                                            )
                                                    {
                                                        weapon.damage = parsed;
                                                    }
                                                });

                                                ui.horizontal(|ui| {
                                                    ui.label("Critical Damage:");
                                                    let mut formula = String::new();
                                                    let critical_damage = weapon
                                                        .critical_damage
                                                        .as_ref()
                                                        .unwrap_or(&weapon.damage);
                                                    critical_damage
                                                        .pretty_print(&mut formula)
                                                        .unwrap();
                                                    if ui
                                                        .add(
                                                            egui::TextEdit::singleline(
                                                                &mut formula,
                                                            )
                                                            .desired_width(100.0),
                                                        )
                                                        .changed()
                                                        && let Ok(parsed) =
                                                            antikythera::roll_parser::parse_roll(
                                                                &formula,
                                                            )
                                                    {
                                                        weapon.critical_damage = Some(parsed);
                                                    }
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Attack Bonus:");
                                                    ui.add(
                                                        egui::DragValue::new(
                                                            &mut weapon.attack_bonus,
                                                        )
                                                        .speed(1)
                                                        .range(-10..=10),
                                                    );
                                                });
                                                ui.horizontal(|ui| {
                                                    ui.label("Range:");
                                                    if let Some(range) = &mut weapon.range {
                                                        ui.add(
                                                            egui::DragValue::new(range)
                                                                .speed(5)
                                                                .range(0..=1000),
                                                        );
                                                    } else {
                                                        ui.label("Melee");
                                                    }
                                                });
                                            });
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
                                                    ui.checkbox(
                                                        &mut armor.stealth_disadvantage,
                                                        "",
                                                    );
                                                });
                                            });
                                    }
                                    _ => {
                                        ui.label("No additional details for this item type.");
                                    }
                                }
                            });
                    }
                });
            });
        });
    }
}
