use antikythera::prelude::*;
use eframe::egui;
use mlua::prelude::*;

pub struct ScriptInterface {
    lua: Lua,
    query: String,
    externals_only: bool,
    script_error: Option<String>,
}

impl Default for ScriptInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptInterface {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            query: String::from(
                r#"function query(state)
    return state.turn > 10
end"#,
            ),
            externals_only: true,
            script_error: None,
        }
    }

    pub fn run_outcome_probability_query(
        &self,
        state_tree: &StateTree,
        query: &str,
        externals_only: bool,
    ) -> anyhow::Result<f64> {
        let query = ScriptProbabilityQuery {
            lua: &self.lua,
            condition: query.to_string(),
            externals_only,
        };
        let result = query.query(state_tree)?;
        Ok(result)
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        state_tree: &Option<StateTree>,
        metrics: &mut Vec<super::Metric>,
    ) {
        ui.label(
            "Enter a Lua function that takes a state and returns true or false. The query will compute the probability of the function returning true.",
        );
        ui.add(egui::TextEdit::multiline(&mut self.query).code_editor());
        ui.checkbox(&mut self.externals_only, "Run on outcomes only");
        if ui.button("Run Query").clicked()
            && let Some(state_tree) = state_tree
        {
            match self.run_outcome_probability_query(state_tree, &self.query, self.externals_only) {
                Ok(probability) => {
                    metrics.push(super::Metric {
                        query_name: if self.externals_only {
                            format!("Terminal-State Probability of:\n{}", self.query)
                        } else {
                            format!("State Probability of:\n{}", self.query)
                        },
                        result: format!("{:.4}%", probability * 100.0),
                    });

                    self.script_error = None;
                }
                Err(e) => {
                    self.script_error = Some(format!("Error running query: {}", e));
                }
            }
        }

        if let Some(error) = &self.script_error {
            ui.colored_label(egui::Color32::RED, error);
        }
    }
}

pub struct ScriptProbabilityQuery<'a> {
    lua: &'a Lua,
    pub condition: String,
    pub externals_only: bool,
}

impl Query for ScriptProbabilityQuery<'_> {
    type Output = f64;

    fn query(&self, state_tree: &StateTree) -> anyhow::Result<Self::Output> {
        let mut total_states = 0;
        let mut count = 0;
        self.lua.load(&self.condition).exec()?;
        let globals = self.lua.globals();
        let func: LuaFunction = globals.get("query")?;

        let mut error = None;

        state_tree.visit_states(self.externals_only, |state, hits| {
            let lua_state = match self.lua.create_userdata(LuaState(state.clone())) {
                Ok(ud) => ud,
                Err(e) => {
                    error = Some(anyhow::anyhow!("Error creating Lua state: {}", e));
                    return false;
                }
            };
            let result: bool = match func.call((lua_state,)) {
                Ok(res) => res,
                Err(e) => {
                    error = Some(anyhow::anyhow!("Error calling Lua function: {}", e));
                    return false;
                }
            };
            if result {
                count += hits;
            }
            total_states += hits;

            self.lua.gc_collect().ok();
            self.lua.gc_collect().ok();

            true
        });

        if let Some(e) = error {
            return Err(e);
        }

        let result = if total_states > 0 {
            count as f64 / total_states as f64
        } else {
            0.0
        };

        Ok(result)
    }
}

pub struct LuaState(pub State);

impl LuaUserData for LuaState {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("turn", |_, this| Ok(this.0.turn));
        fields.add_field_method_get("actors", |lua, this| {
            let table = lua.create_table()?;
            for (id, actor) in &this.0.actors {
                table.set(id.0, LuaActor(actor.clone()))?;
            }
            Ok(table)
        });
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("actor_id", |_, this, name: String| {
            for actor in this.0.actors.values() {
                if actor.name == name {
                    return Ok(LuaActor(actor.clone()));
                }
            }
            Err(LuaError::RuntimeError(format!(
                "No actor found with name '{}'",
                name
            )))
        });

        methods.add_method(
            "actor_alive",
            |_, this, actor_id: LuaValue| match actor_id {
                LuaValue::Integer(id) => {
                    let actor_id = ActorId(id as u32);
                    if let Some(actor) = this.0.get_actor(actor_id) {
                        Ok(actor.is_alive())
                    } else {
                        Err(LuaError::RuntimeError(format!(
                            "No actor found with ID '{}'",
                            id
                        )))
                    }
                }
                LuaValue::String(name) => {
                    for actor in this.0.actors.values() {
                        if actor.name == name.to_string_lossy() {
                            return Ok(actor.is_alive());
                        }
                    }
                    Err(LuaError::RuntimeError(format!(
                        "No actor found with name '{}'",
                        name.to_string_lossy()
                    )))
                }
                _ => Err(LuaError::RuntimeError(
                    "actor_id must be an integer ID or string name".to_string(),
                )),
            },
        );
    }
}

pub struct LuaActor(pub Actor);

impl LuaUserData for LuaActor {
    fn add_fields<F: LuaUserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.0.id.0));
        fields.add_field_method_get("name", |_, this| Ok(this.0.name.clone()));
        fields.add_field_method_get("hp", |_, this| Ok(this.0.health));
        fields.add_field_method_get("max_health", |_, this| Ok(this.0.max_health));
        fields.add_field_method_get("group", |_, this| Ok(this.0.group));
    }

    fn add_methods<M: LuaUserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("is_alive", |_, this, ()| Ok(this.0.is_alive()));
        methods.add_method("is_unconscious", |_, this, ()| Ok(this.0.is_unconscious()));
        methods.add_method("is_dead", |_, this, ()| Ok(this.0.is_dead()));
    }
}
