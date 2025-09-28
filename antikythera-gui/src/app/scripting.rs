use antikythera::prelude::*;
use mlua::prelude::*;

pub mod analysis;
pub mod simulation;

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
