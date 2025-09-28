use antikythera::prelude::*;
use mlua::prelude::*;

use crate::app::scripting::LuaState;

const DEFAULT_QUERY_SCRIPT: &str = r#"function query(state)
    -- Example: check if the actor named "Hero" is alive
    return state:actor_alive("Hero")
end"#;

pub struct AnalysisScriptInterface {
    lua: Lua,
    pub query: String,
    pub last_saved_query: Option<String>,
    pub externals_only: bool,
    pub script_error: Option<String>,
    pub metrics: Vec<(String, f64)>,
}

impl Default for AnalysisScriptInterface {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisScriptInterface {
    pub fn new() -> Self {
        Self {
            lua: Lua::new(),
            query: String::from(DEFAULT_QUERY_SCRIPT),
            last_saved_query: Some(String::from(DEFAULT_QUERY_SCRIPT)),
            externals_only: true,
            script_error: None,
            metrics: Vec::new(),
        }
    }

    pub fn has_unsaved_changes(&self) -> bool {
        if let Some(last) = &self.last_saved_query {
            &self.query != last
        } else {
            true
        }
    }

    fn reset_lua(&mut self) {
        self.lua = Lua::new();
        self.script_error = None;

        // insert an empty table for metrics
        let globals = self.lua.globals();
        if let Err(e) = globals.set("metrics", self.lua.create_table().unwrap()) {
            self.script_error = Some(format!("Error creating metrics table: {}", e));
        }

        self.metrics.clear();
    }

    pub fn run_outcome_probability_query(&mut self, state_tree: &StateTree) -> anyhow::Result<f64> {
        self.reset_lua();

        let query = ScriptProbabilityQuery {
            lua: &self.lua,
            condition: self.query.clone(),
            externals_only: self.externals_only,
        };
        let result = query.query(state_tree)?;
        Ok(result)
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
