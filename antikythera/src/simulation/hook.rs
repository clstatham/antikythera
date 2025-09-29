use crate::{prelude::*, rules::actions::ActionTaken};

#[allow(unused)]
pub trait Hook: Send + Sync {
    fn on_integration_start(&mut self, initial_state: &State) {}
    fn on_combat_start(&mut self, state: &State) {}
    fn on_turn_start(&mut self, state: &State, actor_id: ActorId, turn: u64) {}
    fn on_advance_initiative(&mut self, state: &State, actor_id: ActorId) {}
    fn on_action_executed(&mut self, state: &State, action: &ActionTaken) {}
    fn on_turn_end(&mut self, state: &State, actor_id: ActorId, turn: u64) {}
    fn on_combat_end(&mut self, state: &State) {}
    fn on_integration_end(&mut self) {}

    fn metrics(&self) -> Vec<(String, f64)> {
        vec![]
    }
}
