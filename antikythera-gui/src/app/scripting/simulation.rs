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
            script,
            script_rx,
            script_error_tx,
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

macro_rules! lua_delegate {
    ($self:expr, $func:ident, $($arg:expr),*) => {
        if let Ok(globals) = $self.lua.globals().get::<LuaFunction>(stringify!($func))
            && let Err(e) = globals.call::<()>(($($arg),*))
        {
            log::error!("Error in {}: {}", stringify!($func), e);
            let _ = $self
                .script_error_tx
                .send(format!("Error in {}: {}", stringify!($func), e));
        }
    };
}

impl Hook for LuaHook {
    fn on_integration_start(&mut self, initial_state: &State) {
        self.reload_script();
        lua_delegate!(self, on_integration_start, LuaState(initial_state.clone()));
    }

    fn on_combat_start(&mut self, state: &State) {
        lua_delegate!(self, on_combat_start, LuaState(state.clone()));
    }

    fn on_turn_start(&mut self, state: &State, actor_id: ActorId, turn: u64) {
        lua_delegate!(
            self,
            on_turn_start,
            LuaState(state.clone()),
            actor_id.0 as i64,
            turn
        );
    }

    fn on_advance_initiative(&mut self, state: &State, actor_id: ActorId) {
        lua_delegate!(
            self,
            on_advance_initiative,
            LuaState(state.clone()),
            actor_id.0 as i64
        );
    }

    fn on_action_executed(&mut self, state: &State, action: &ActionTaken) {
        let action = self.lua.to_value(&action).unwrap_or(LuaValue::Nil);
        lua_delegate!(self, on_action_executed, LuaState(state.clone()), action);
    }

    fn on_turn_end(&mut self, state: &State, actor_id: ActorId, turn: u64) {
        lua_delegate!(
            self,
            on_turn_end,
            LuaState(state.clone()),
            actor_id.0 as i64,
            turn
        );
    }

    fn on_combat_end(&mut self, state: &State) {
        lua_delegate!(self, on_combat_end, LuaState(state.clone()));
    }

    fn on_integration_end(&mut self) {
        lua_delegate!(self, on_integration_end,);
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
