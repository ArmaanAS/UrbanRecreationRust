use std::{collections::HashMap, fs::File, path::Path, sync::Mutex};

use colored::Colorize;
use lazy_static::lazy_static;
use nohash_hasher::BuildNoHashHasher;
use regex::Regex;
use serde::{de::IntoDeserializer, Deserialize, Deserializer};
use serde_repr::Deserialize_repr;
// use simd_json::from_reader;
use serde_json::from_reader;
use tinyvec::ArrayVec;

use crate::{
    battle::BattleData,
    game::RoundWin,
    modifiers::{EventTime, Modifier},
    types::Clan,
};

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

lazy_static! {
    // pub static ref ABILITIES: HashMap<u32, Cow<'static, Ability>, BuildNoHashHasher<u32>> = {
    pub static ref ABILITIES: HashMap<u32, Ability, BuildNoHashHasher<u32>> = {
        let data_file =
            File::open(Path::new("./assets/compiled.json")).unwrap();
        from_reader(data_file).expect("Error while reading JSON file")
        // from_reader::<_, HashMap<u32, Ability>>(data_file)
        //     .expect("Error while reading JSON file")
        //     .into_iter()
        //     .map(|(k, v)| (k, Cow::Owned(v)))
        //     .collect()
    };
    pub static ref CONDITION_CLANS: Mutex<HashMap<u8, Vec<Clan>>> = Mutex::new(HashMap::new());
    pub static ref CLANS_REGEX: Regex = Regex::new(r"\[[Cc]lan:(\d+)\]").unwrap();
}

#[derive(Clone, Copy, Debug, Deserialize_repr, PartialEq)]
#[repr(usize)]
pub enum AbilityType {
    Global = 1,
    Ability = 2,
    Bonus = 3,
    GlobalAbility = 4,
    GlobalBonus = 5,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Ability {
    // pub ability: String,
    pub ability_type: AbilityType,
    pub modifiers: ArrayVec<[Option<Modifier>; 2]>,
    pub conditions: ArrayVec<[Option<Condition>; 3]>,
    #[serde(default)]
    pub delayed: bool,
    #[serde(default)]
    pub won: bool,
    #[serde(default)]
    pub remove: bool,
}

impl Ability {
    #[inline]
    pub fn event_time(&self) -> EventTime {
        self.modifiers[0].as_ref().unwrap().event_time()
    }
    pub fn can_apply(&mut self, data: &BattleData) -> bool {
        if (self.ability_type == AbilityType::GlobalAbility
            || self.ability_type == AbilityType::GlobalBonus)
            && !self.won
        {
            if data.player.borrow().won == RoundWin::LOSE {
                self.remove = true;
                return false;
            } else {
                self.won = true;
            }
        }

        if self.delayed {
            self.delayed = false;
            return false;
        }

        for cond in self.conditions.iter() {
            if !cond.as_ref().unwrap().is_met(data) {
                println!("{}: {:?}", "Condition not met".red(), cond);
                return false;
            }
            println!("{}: {:?}", "Condition met".green(), cond);
        }

        match self.ability_type {
            AbilityType::Ability | AbilityType::GlobalAbility => {
                !data.card.borrow().ability.is_blocked()
            }
            AbilityType::Bonus | AbilityType::GlobalBonus => !data.card.borrow().bonus.is_blocked(),
            _ => true,
        }
    }
    pub fn apply(&mut self, data: &BattleData) -> Option<Ability> {
        let mut ability: Option<Ability> = None;
        if self.can_apply(data) {
            for modifier in self.modifiers.iter_mut() {
                println!("{}: {:?}", "Applying".yellow(), modifier.as_ref().unwrap());
                ability = modifier.as_mut().unwrap().apply(data);
            }
        }
        ability
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
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
    Infiltrate(u8),
    Versus(u8),
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

            let mut cache = CONDITION_CLANS.lock().unwrap();
            let key = cache.len() as u8;
            cache.insert(key, clans);

            if s.starts_with("Versus") {
                Ok(Condition::Versus(key))
            } else {
                Ok(Condition::Infiltrate(key))
            }
        } else {
            Condition::deserialize(s.into_deserializer())
        }
    }
}

impl Condition {
    #[inline]
    pub fn is_met(&self, data: &BattleData) -> bool {
        match self {
            Condition::Defeat => data.player.borrow().won == RoundWin::LOSE,
            // Condition::Night => round.day == false,
            // Condition::Day => round.day == true,
            Condition::Night | Condition::Day => true,
            Condition::Courage => data.first,
            Condition::Revenge => data.player.borrow().won_previous == RoundWin::LOSE,
            Condition::Confidence => data.player.borrow().won_previous == RoundWin::WIN,
            Condition::Reprisal => !data.first,
            Condition::Killshot => {
                data.card.borrow().attack.value >= data.opp_card.borrow().attack.value * 2
            }
            Condition::Backlash => data.player.borrow().won == RoundWin::WIN,
            Condition::Reanimate => {
                data.player.borrow().won == RoundWin::LOSE && data.player.borrow().life == 0
            }
            Condition::Stop => data.card.borrow().ability.cancelled != 0,
            Condition::Symmetry => data.card.borrow().index == data.opp_card.borrow().index,
            Condition::Asymmetry => data.card.borrow().index != data.opp_card.borrow().index,
            Condition::Infiltrate(key) => {
                let hand = data.hand;
                println!("{}", hand.oculus_clan);
                if hand.oculus_clan == Clan::None {
                    false
                } else {
                    let clans = &CONDITION_CLANS.lock().unwrap()[key];
                    clans.contains(&hand.oculus_clan)
                }
            }
            Condition::Versus(key) => {
                let opp_hand = data.opp_hand;
                for card in opp_hand.cards.iter() {
                    let clans = &CONDITION_CLANS.lock().unwrap()[key];
                    if clans.contains(&card.borrow().clan()) {
                        return true;
                    }
                }

                false
            }
            _ => true,
        }
    }
}
