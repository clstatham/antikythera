use antikythera::prelude::*;
use mlua::prelude::*;

use crate::app::scripting::LuaState;

pub struct LuaHookHandle {
    pub script_tx: crossbeam_channel::Sender<String>,
    pub script_error_rx: crossbeam_channel::Receiver<String>,
}

pub struct LuaHook {
    lua: Lua,
    pub script: String,
    script_rx: crossbeam_channel::Receiver<String>,
    script_error_tx: crossbeam_channel::Sender<String>,
}

impl LuaHook {
    pub fn new(script: String) -> (Self, LuaHookHandle) {
        let (script_tx, script_rx) = crossbeam_channel::unbounded();
        let (script_error_tx, script_error_rx) = crossbeam_channel::unbounded();
        let mut this = Self {
            lua: Lua::new(),
            script_rx,
            script_error_tx,
            script,
        };
        this.reset_lua();

        if !this.script.is_empty()
            && let Err(e) = this.lua.load(&this.script).exec()
        {
            let _ = this
                .script_error_tx
                .send(format!("Error loading Lua script: {}", e));
        }

        (
            this,
            LuaHookHandle {
                script_tx,
                script_error_rx,
            },
        )
    }

    fn reset_lua(&mut self) {
        self.lua = Lua::new();

        // insert an empty table for metrics
        let globals = self.lua.globals();
        if let Err(e) = globals.set("metrics", self.lua.create_table().unwrap()) {
            let _ = self
                .script_error_tx
                .send(format!("Error creating metrics table: {}", e));
        }
    }

    fn reload_script(&mut self) {
        let mut script_changed = false;
        while let Ok(new_script) = self.script_rx.try_recv() {
            self.script = new_script;
            script_changed = true;
        }

        if !script_changed {
            return;
        }

        self.reset_lua();

        if !self.script.is_empty()
            && let Err(e) = self.lua.load(&self.script).exec()
        {
            log::error!("Error loading Lua script: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error loading Lua script: {}", e));
        }
    }
}

impl Hook for LuaHook {
    fn on_integration_start(&mut self, initial_state: &State) {
        self.reload_script();
        if let Ok(globals) = self
            .lua
            .globals()
            .get::<LuaFunction>("on_integration_start")
            && let Err(e) = globals.call::<()>((LuaState(initial_state.clone()),))
        {
            log::error!("Error in on_integration_start: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_integration_start: {}", e));
        }
    }

    fn on_combat_start(&mut self, state: &State) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_combat_start")
            && let Err(e) = globals.call::<()>(LuaState(state.clone()))
        {
            log::error!("Error in on_combat_start: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_combat_start: {}", e));
        }
    }

    fn on_turn_start(&mut self, state: &State, actor_id: ActorId, turn: u64) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_turn_start")
            && let Err(e) = globals.call::<()>((LuaState(state.clone()), actor_id.0 as i64, turn))
        {
            log::error!("Error in on_turn_start: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_turn_start: {}", e));
        }
    }

    fn on_action_executed(&mut self, state: &State, action: &ActionTaken) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_action_executed") {
            let action = self.lua.to_value(action).unwrap_or(LuaValue::Nil);
            if let Err(e) = globals.call::<()>((LuaState(state.clone()), action)) {
                log::error!("Error in on_action_executed: {}", e);
                let _ = self
                    .script_error_tx
                    .send(format!("Error in on_action_executed: {}", e));
            }
        }
    }

    fn on_turn_end(&mut self, state: &State, actor_id: ActorId, turn: u64) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_turn_end")
            && let Err(e) = globals.call::<()>((LuaState(state.clone()), actor_id.0 as i64, turn))
        {
            log::error!("Error in on_turn_end: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_turn_end: {}", e));
        }
    }

    fn on_combat_end(&mut self, state: &State) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_combat_end")
            && let Err(e) = globals.call::<()>(LuaState(state.clone()))
        {
            log::error!("Error in on_combat_end: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_combat_end: {}", e));
        }
    }

    fn on_integration_end(&mut self) {
        if let Ok(globals) = self.lua.globals().get::<LuaFunction>("on_integration_end")
            && let Err(e) = globals.call::<()>(())
        {
            log::error!("Error in on_integration_end: {}", e);
            let _ = self
                .script_error_tx
                .send(format!("Error in on_integration_end: {}", e));
        }
    }

    fn metrics(&self) -> Vec<(String, f64)> {
        let mut result = Vec::new();
        if let Ok(globals) = self.lua.globals().get::<LuaTable>("metrics") {
            for (key, value) in globals.pairs::<String, f64>().flatten() {
                result.push((key, value));
            }
        } else {
            log::error!("Error accessing metrics table");
        }
        result
    }
}
