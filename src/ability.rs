use std::{collections::HashMap, fs::File, path::Path};

use colored::Colorize;
use lazy_static::lazy_static;
use regex::Regex;
use serde::{de::IntoDeserializer, Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
use simd_json::from_reader;

pub static mut PRINT: bool = true;
macro_rules! println {
    ($($rest:tt)*) => {
        unsafe {
            if PRINT {
                std::println!($($rest)*)
            }
        }
    }
}

use crate::{
    battle::BattleData,
    game::RoundWin,
    modifiers::{EventTime, Modifier},
    types::Clan,
};

lazy_static! {
    pub static ref ABILITIES: HashMap<u32, Ability> = {
        let data_file =
            File::open(Path::new("./assets/compiled.json")).expect("file should open read only");
        from_reader(data_file).expect("Error while reading JSON file")
    };
    pub static ref CLANS_REGEX: Regex = Regex::new(r"\[[Cc]lan:(\d+)\]").unwrap();
}

#[derive(Clone, Debug, Deserialize_repr, PartialEq)]
#[repr(usize)]
pub enum AbilityType {
    None = 0,
    Global = 1,
    Ability = 2,
    Bonus = 3,
    GlobalAbility = 4,
    GlobalBonus = 5,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Ability {
    pub ability: String,
    pub ability_type: AbilityType,
    pub modifiers: Vec<Modifier>,
    pub conditions: Vec<Condition>,
    pub delayed: Option<bool>,
    pub won: Option<bool>,
    pub remove: Option<()>,
}

impl Ability {
    pub fn event_time(&self) -> EventTime {
        self.modifiers[0].event_time()
    }
    pub fn can_apply(&mut self, data: &BattleData) -> bool {
        if (self.ability_type == AbilityType::GlobalAbility
            || self.ability_type == AbilityType::GlobalBonus)
            && self.won == None
        {
            if data.player.borrow().won == RoundWin::LOSE {
                // data.events.borrow_mut().remove_global(self);
                self.remove = Some(());
                return false;
            } else {
                self.won = Some(true);
            }
        }

        if self.delayed == Some(true) {
            self.delayed = Some(false);
            return false;
        }

        for cond in self.conditions.iter() {
            if !cond.is_met(data) {
                println!("{}: {:?}", "Condition not met".red(), cond);
                return false;
            }
            println!("{}: {:?}", "Condition met".green(), cond);
        }

        match self.ability_type {
            AbilityType::Ability | AbilityType::GlobalAbility => {
                !data.card.borrow().ability.attr.is_blocked()
            }
            AbilityType::Bonus | AbilityType::GlobalBonus => {
                !data.card.borrow().bonus.attr.is_blocked()
            }
            _ => true,
        }
    }
    pub fn apply(&mut self, data: &BattleData) -> Option<Ability> {
        let mut ability: Option<Ability> = None;
        if self.can_apply(data) {
            for modifier in self.modifiers.iter_mut() {
                println!("{}: {:?}", "Applying".yellow(), modifier);
                ability = modifier.apply(data);
            }
        }
        ability
    }
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(remote = "Condition")]
pub enum Condition {
    Courage,
    Defeat,
    Brawl,
    Growth,
    Confidence,
    Degrowth,
    #[serde(rename = "Victory Or Defeat")]
    VictoryOrDefeat,
    Equalizer,
    Support,
    Team,
    Symmetry,
    Revenge,
    Reprisal,
    Day,
    Night,
    Killshot,
    Backlash,
    Asymmetry,
    Reanimate,
    Stop,
    // StopBonus,
    Infiltrate(Vec<Clan>),
    Versus(Vec<Clan>),
    // Other(String),
    #[serde(other)]
    None,
}

impl<'de> Deserialize<'de> for Condition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.ends_with("]") {
            let clans = CLANS_REGEX
                .captures_iter(s.as_str())
                .map(|m| Clan::from(*&m[1].parse::<u8>().unwrap()))
                .collect::<Vec<Clan>>();
            if s.starts_with("Versus") {
                Ok(Condition::Versus(clans))
            } else {
                Ok(Condition::Infiltrate(clans))
            }
        } else {
            Condition::deserialize(s.into_deserializer())
        }
    }
}

impl Condition {
    pub fn is_met(&self, data: &BattleData) -> bool {
        let player = data.player.borrow();
        let card = data.card.borrow();
        let opp_card = data.opp_card.borrow();
        let round = &data.round;
        let hand = data.round.hand;
        let opp_hand = data.round.opp_hand;
        match self {
            Condition::Defeat => player.won == RoundWin::LOSE,
            // Condition::Night => round.day == false,
            // Condition::Day => round.day == true,
            Condition::Night | Condition::Day => true,
            Condition::Courage => round.first,
            Condition::Revenge => player.won_previous == RoundWin::LOSE,
            Condition::Confidence => player.won_previous == RoundWin::WIN,
            Condition::Reprisal => !round.first,
            Condition::Killshot => card.attack.value >= opp_card.attack.value * 2,
            Condition::Backlash => player.won == RoundWin::WIN,
            Condition::Reanimate => player.won == RoundWin::LOSE && player.life == 0,
            Condition::Stop => card.ability.attr.cancelled != 0,
            Condition::Symmetry => card.index == opp_card.index,
            Condition::Asymmetry => card.index != opp_card.index,
            Condition::Infiltrate(clans) => {
                println!("{}", hand.oculus_clan);
                if hand.oculus_clan == Clan::None {
                    false
                } else {
                    clans.contains(&hand.oculus_clan)
                }
            }
            Condition::Versus(clans) => {
                for card in opp_hand.cards.iter() {
                    if clans.contains(&card.borrow().clan) {
                        return true;
                    }
                }

                false
            }
            _ => true,
        }
    }
}
