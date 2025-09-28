use crate::{
    simulation::{
        action_evaluator::ActionEvaluator,
        logging::{LogEntry, SimulationLog},
        policy::RandomPolicy,
        state::State,
    },
    statistics::roller::Roller,
    utils::ProtectedCell,
};

#[derive(Debug)]
pub struct Executor {
    pub roller: Roller,
    pub state: ProtectedCell<State>,
    pub log: SimulationLog,
    pub evaluator: ActionEvaluator,
    pub policy: RandomPolicy,
}

impl Executor {
    pub fn new(roller: Roller, state: State) -> Self {
        Self {
            roller,
            state: ProtectedCell::new(state),
            log: SimulationLog::default(),
            evaluator: ActionEvaluator,
            policy: RandomPolicy,
        }
    }

    pub fn take_log(&mut self) -> SimulationLog {
        std::mem::take(&mut self.log)
    }

    pub fn save_log(&self, path: &std::path::Path) -> anyhow::Result<()> {
        self.log.save(path)
    }

    pub fn log(&mut self, entry: LogEntry) -> anyhow::Result<()> {
        self.log.log(entry, &self.state);
        Ok(())
    }

    pub fn extend_log(
        &mut self,
        entries: impl IntoIterator<Item = LogEntry>,
    ) -> anyhow::Result<()> {
        for entry in entries {
            self.log(entry)?;
        }
        Ok(())
    }

    pub fn clear_log(&mut self) -> anyhow::Result<()> {
        self.log = SimulationLog::default();
        Ok(())
    }
}
