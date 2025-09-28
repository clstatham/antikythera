-- Hook to track average number of turns per combat

turn_counter = 0
combat_counter = 0
metrics = {}

function on_integration_start(initial_state)
    -- Initialize any state or metrics here
    turn_counter = 0
    combat_counter = 0
    metrics["avg_turns"] = 0
end

function on_combat_start(state)
    -- Called at the start of each combat
    combat_counter = combat_counter + 1
    turn_counter = 0
end

function on_turn_start(state, actor_id, turn)
    -- Called at the start of each turn
    turn_counter = turn_counter + 1
end

function on_combat_end(state)
    -- Called at the end of each combat
    metrics["avg_turns"] = metrics["avg_turns"] + turn_counter
end

function on_integration_end()
    -- Finalize metrics here
    metrics["avg_turns"] = metrics["avg_turns"] / combat_counter
end
