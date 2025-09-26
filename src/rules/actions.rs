use serde::{Deserialize, Serialize};

use crate::{
    rules::{
        actor::ActorId,
        dice::RollSettings,
        items::ItemId,
        saves::SavingThrow,
        spells::{SpellId, SpellTarget},
    },
    simulation::state::SimulationState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionType {
    Wait,
    UnarmedStrike,
    Attack,
    CastSpell,
    UseItem,
    Dash,
    Disengage,
    Dodge,
    Help,
    Hide,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Action {
    Wait,
    UnarmedStrike(UnarmedStrikeAction),
    Attack(AttackAction),
    CastSpell(CastSpellAction),
    UseItem(UseItemAction),
    Dash,
    Disengage,
    Dodge,
    Help(HelpAction),
    Hide,
    // todo:
    // Ready(ReadyAction),
    // Search(SearchAction),
    // UseObject(UseObjectAction),
}

impl Action {
    pub fn action_type(&self) -> ActionType {
        match self {
            Action::Wait => ActionType::Wait,
            Action::UnarmedStrike(_) => ActionType::UnarmedStrike,
            Action::Attack(_) => ActionType::Attack,
            Action::CastSpell(_) => ActionType::CastSpell,
            Action::UseItem(_) => ActionType::UseItem,
            Action::Dash => ActionType::Dash,
            Action::Disengage => ActionType::Disengage,
            Action::Dodge => ActionType::Dodge,
            Action::Help(_) => ActionType::Help,
            Action::Hide => ActionType::Hide,
        }
    }

    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &SimulationState,
    ) -> std::fmt::Result {
        match self {
            Action::Wait => write!(f, "Wait"),
            Action::UnarmedStrike(action) => {
                write!(f, "Unarmed Strike at target ")?;
                action.target.pretty_print(f, state)?;
                Ok(())
            }
            Action::Attack(action) => {
                write!(f, "Attack target ")?;
                action.target.pretty_print(f, state)?;
                write!(f, " with weapon ")?;
                action.weapon_used.pretty_print(f, state)?;

                Ok(())
            }
            Action::CastSpell(action) => {
                write!(f, "Cast spell {:?} on targets ", action.spell_used)?;
                for (i, target) in action.targets.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    target.pretty_print(f, state)?;
                }
                Ok(())
            }
            Action::UseItem(action) => {
                write!(f, "Use item {:?}", action.item_used)?;
                if let Some(target) = &action.target {
                    write!(f, " on target ")?;
                    target.pretty_print(f, state)?;
                }
                Ok(())
            }
            Action::Dash => write!(f, "Dash"),
            Action::Disengage => write!(f, "Disengage"),
            Action::Dodge => write!(f, "Dodge"),
            Action::Help(action) => {
                write!(f, "Help target ")?;
                action.target.pretty_print(f, state)?;
                Ok(())
            }
            Action::Hide => write!(f, "Hide"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnarmedStrikeAction {
    pub target: ActorId,
    pub attack_roll_settings: RollSettings,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackAction {
    pub weapon_used: ItemId,
    pub target: ActorId,
    pub attack_roll_settings: RollSettings,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CastSpellAction {
    pub spell_used: SpellId,
    pub targets: Vec<SpellTarget>,
    pub save_dc: Option<u32>,
    pub save_type: Option<SavingThrow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UseItemAction {
    pub item_used: ItemId,
    pub target: Option<ActorId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HelpAction {
    pub target: ActorId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionEconomyUsage {
    Action,
    BonusAction,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionEconomy {
    pub action_used: bool,
    pub bonus_action_used: bool,
    pub reaction_used: bool,
    pub free_actions_used: u32,
    pub movement_used: u32,
}

impl ActionEconomy {
    pub fn reset(&mut self) {
        self.action_used = false;
        self.bonus_action_used = false;
        self.reaction_used = false;
        self.free_actions_used = 0;
        self.movement_used = 0;
    }

    pub fn can_take_action(&self, action_type: ActionEconomyUsage) -> bool {
        match action_type {
            ActionEconomyUsage::Action => !self.action_used,
            ActionEconomyUsage::BonusAction => !self.bonus_action_used,
            // ActionType::Reaction => !self.reaction_used,
            // ActionType::FreeAction => true,
        }
    }

    pub fn use_action(&mut self, action_type: ActionEconomyUsage) -> anyhow::Result<()> {
        match action_type {
            ActionEconomyUsage::Action => {
                if self.action_used {
                    anyhow::bail!("Action already used this turn");
                }
                self.action_used = true;
            }
            ActionEconomyUsage::BonusAction => {
                if self.bonus_action_used {
                    anyhow::bail!("Bonus action already used this turn");
                }
                self.bonus_action_used = true;
            } // ActionType::Reaction => {
              //     if self.reaction_used {
              //         anyhow::bail!("Reaction already used this turn");
              //     }
              //     self.reaction_used = true;
              // }
              // ActionType::FreeAction => {
              //     self.free_actions_used += 1;
              // }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ActionTaken {
    pub actor: ActorId,
    pub action: Action,
    pub action_type: ActionEconomyUsage,
}

impl ActionTaken {
    pub fn pretty_print(
        &self,
        f: &mut impl std::fmt::Write,
        state: &SimulationState,
    ) -> std::fmt::Result {
        write!(f, "Actor ")?;
        self.actor.pretty_print(f, state)?;
        write!(f, " takes action: ")?;
        self.action.pretty_print(f, state)?;
        write!(f, " as a {:?}", self.action_type)?;
        Ok(())
    }
}
