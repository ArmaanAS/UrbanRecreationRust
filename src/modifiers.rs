use serde::Deserialize;
use serde_repr::Deserialize_repr;

use crate::{
    ability::{Ability, AbilityType},
    battle::BattleData,
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

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(usize)]
pub enum EventTime {
    START = 0,

    PRE4 = 1,
    PRE3 = 2,
    PRE2 = 3,
    PRE1 = 4,

    POST1 = 5,
    POST2 = 6,
    POST3 = 7,
    POST4 = 8,

    END = 9,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Stat {
    Power,
    Damage,
    Attack,
    Life,
    Pillz,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[serde(untagged)]
#[repr(u8)]
pub enum Per {
    Power = 1,
    Damage = 2,
    Life = 3,
    Pillz = 4,
    Support = 5,
    Brawl = 6,
    Growth = 7,
    Degrowth = 8,
    Equalizer = 9,
    Symmetry = 10,
    Asymmetry = 11,
    OppPower = 12,
    OppDamage = 13,
    OppLife = 14,
    OppPillz = 15,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Cancel {
    Power = 1,
    Damage = 2,
    Attack = 3,
    Ability = 4,
    Bonus = 5,
    Pillz = 6,
    Life = 7,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Copy {
    Power = 1,
    Damage = 2,
    Ability = 3,
    Bonus = 4,
    Infiltrate = 5,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Exchange {
    Power = 1,
    Damage = 2,
    ImposePower = 3,
    ImposeDamage = 4,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Protect {
    Power = 1,
    Damage = 2,
    Attack = 3,
    Ability = 4,
    Bonus = 5,
}

#[derive(Debug, Clone, Copy, Deserialize_repr, PartialEq)]
#[repr(u8)]
pub enum Recover {
    Pillz = 1,
    Life = 2,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct BasicModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    change: i32,
    per: Option<Per>,
    #[serde(rename = "type")]
    stat: Stat,
    opp: bool,
    min: i32,
    max: i32,
    always: bool,
}

impl BasicModifier {
    fn get_multiplier(&self, data: &BattleData) -> u8 {
        if self.per == None {
            return 1;
        }

        let (player, card) = if self.opp {
            (data.opp.borrow(), data.opp_card.borrow())
        } else {
            (data.player.borrow(), data.card.borrow())
        };

        let player_card = data.card.borrow();
        let opp_card = data.opp_card.borrow();
        match self.per.unwrap() {
            Per::Power => card.power.value,
            Per::Damage => card.damage.value,
            Per::Life => player.life,
            Per::Pillz => player.pillz,
            Per::Support => data.hand.card_clan_count(player_card.index),
            Per::Brawl => data.opp_hand.card_clan_count(opp_card.index),
            Per::Growth => 1 + data.round,
            Per::Degrowth => 4 - data.round,
            Per::Equalizer => opp_card.level,
            Per::Symmetry => (player_card.index == opp_card.index) as u8,
            Per::Asymmetry => (player_card.index != opp_card.index) as u8,
            _ => 1,
        }
    }
    fn modify(&self, base: u8, data: &BattleData) -> u8 {
        if (base as i32) < self.min || (base as i32) >= self.max {
            return base as u8;
        }

        let multiplier = self.get_multiplier(data) as i32;
        let change = self.change * multiplier as i32;
        let value = base as i32 + change;
        let squash = value.max(self.min).min(self.max);

        println!("{} => {} >=< {}", base, value, squash);

        squash as u8
    }
}

impl BasicModifier {
    fn can_apply(&self, data: &BattleData) -> bool {
        if self.always {
            return true;
        }
        if self.win == Some(true) && !data.card.borrow().won {
            return false;
        }

        let card = data.card.borrow();
        if self.opp {
            let opp_card = data.opp_card.borrow();
            println!("opp_card = {:?}", opp_card.life);
            match self.stat {
                Stat::Power => !opp_card.power.attr.is_protected() && !card.power.attr.is_blocked(),
                Stat::Damage => {
                    !opp_card.damage.attr.is_protected() && !card.damage.attr.is_blocked()
                }
                Stat::Attack => {
                    !opp_card.attack.attr.is_protected() && !card.attack.attr.is_blocked()
                }
                Stat::Life => {
                    !opp_card.life.is_protected()
                        && !card.life.is_blocked()
                        && data.player.borrow().life > 0
                }
                Stat::Pillz => !opp_card.pillz.is_protected() && !card.pillz.is_blocked(),
            }
        } else {
            // println!("card = {:#?}", card);
            match self.stat {
                Stat::Power => !card.power.attr.is_blocked(),
                Stat::Damage => !card.damage.attr.is_blocked(),
                Stat::Attack => !card.attack.attr.is_blocked(),
                Stat::Life => !card.life.is_blocked() && data.player.borrow().life > 0,
                Stat::Pillz => !card.pillz.is_blocked(),
            }
        }
    }
    pub fn apply(&mut self, data: &BattleData) {
        // println!(
        //     "{}, {}",
        //     data.card.borrow().ability.attr.blocked(),
        //     data.card.borrow().bonus.attr.blocked(),
        // );
        if self.can_apply(data) {
            println!("Applying modifier: {:?}", self);

            let (card, player) = if self.opp {
                (data.opp_card, data.opp)
            } else {
                (data.card, data.player)
            };

            match self.stat {
                Stat::Power => {
                    let val = self.modify(card.borrow().power.value, data);
                    card.borrow_mut().power.value = val;
                }
                Stat::Damage => {
                    let val = self.modify(card.borrow().damage.value, data);
                    card.borrow_mut().damage.value = val;
                }
                Stat::Attack => {
                    let val = self.modify(card.borrow().attack.value, data);
                    card.borrow_mut().attack.value = val;
                }
                Stat::Life => {
                    let val = self.modify(player.borrow().life, data);
                    player.borrow_mut().life = val;
                }
                Stat::Pillz => {
                    let val = self.modify(player.borrow().pillz, data);
                    player.borrow_mut().pillz = val;
                }
            }
        } else {
            println!("Can't apply modifier: {:?}", self);
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct CancelModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    cancel: Cancel,
    pub applied: Option<bool>,
}

// impl ApplyModifier for CancelModifier {
impl CancelModifier {
    pub fn apply(&mut self, data: &BattleData) {
        self.applied = Some(true);
        let mut opp_card = data.opp_card.borrow_mut();
        match self.cancel {
            Cancel::Power => opp_card.power.attr.cancel(),
            Cancel::Damage => opp_card.damage.attr.cancel(),
            Cancel::Attack => opp_card.attack.attr.cancel(),
            Cancel::Pillz => opp_card.pillz.cancel(),
            Cancel::Life => opp_card.life.cancel(),
            // Cancel::Ability => opp_card.ability.attr.cancel(),
            // Cancel::Bonus => opp_card.bonus.attr.cancel(),
            Cancel::Ability => opp_card.ability.cancel(),
            Cancel::Bonus => opp_card.bonus.cancel(),
        }
    }
    pub fn undo(&mut self, data: &BattleData) {
        self.applied = Some(false);
        let mut opp_card = data.opp_card.borrow_mut();
        match self.cancel {
            Cancel::Power => opp_card.power.attr.remove_cancel(),
            Cancel::Damage => opp_card.damage.attr.remove_cancel(),
            Cancel::Attack => opp_card.attack.attr.remove_cancel(),
            Cancel::Pillz => opp_card.pillz.remove_cancel(),
            Cancel::Life => opp_card.life.remove_cancel(),
            // Cancel::Ability => opp_card.ability.attr.remove_cancel(),
            // Cancel::Bonus => opp_card.bonus.attr.remove_cancel(),
            Cancel::Ability => opp_card.ability.remove_cancel(),
            Cancel::Bonus => opp_card.bonus.remove_cancel(),
        }
    }
    // fn can_apply(&self, data: &BattleData) -> bool {
    //     true
    // }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct CopyModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    copy: Copy,
}

impl CopyModifier {
    pub fn apply(&mut self, data: &BattleData) -> Option<Ability> {
        let mut card = data.card.borrow_mut();
        let opp_card = data.opp_card.borrow();
        match self.copy {
            Copy::Power => card.power.value = opp_card.power.base,
            Copy::Damage => card.damage.value = opp_card.damage.base,
            Copy::Ability => {
                card.bonus = opp_card.ability;
                card.bonus_id = opp_card.ability_id;

                let mut bonus = card.get_bonus();
                match &bonus.ability_type {
                    AbilityType::Ability => bonus.ability_type = AbilityType::Bonus,
                    AbilityType::GlobalAbility => bonus.ability_type = AbilityType::GlobalBonus,
                    AbilityType::Global => return None,
                    _ => (),
                }

                return Some(bonus);
            }
            Copy::Bonus => {
                card.ability = opp_card.bonus;
                card.ability_id = opp_card.bonus_id;

                let mut ability = card.get_ability();
                match &ability.ability_type {
                    AbilityType::Bonus => ability.ability_type = AbilityType::Ability,
                    AbilityType::GlobalBonus => ability.ability_type = AbilityType::GlobalAbility,
                    AbilityType::Global => return None,
                    _ => (),
                }

                return Some(ability);
            }
            Copy::Infiltrate => {
                let clan = data.hand.oculus_clan;
                if clan != Clan::None {
                    for (i, clan_card) in data.hand.cards.iter().enumerate() {
                        if i != card.index {
                            let clan_card = clan_card.borrow();
                            if clan_card.clan() == clan {
                                card.bonus = clan_card.bonus;
                                card.bonus_id = clan_card.bonus_id;

                                return Some(card.get_bonus());
                            }
                        }
                    }
                }
            }
        };
        None
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct ExchangeModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    ex: Exchange,
}

impl ExchangeModifier {
    pub fn apply(&mut self, data: &BattleData) {
        let mut card = data.card.borrow_mut();
        let mut opp_card = data.opp_card.borrow_mut();
        match self.ex {
            Exchange::Power => {
                if !card.power.attr.is_blocked() {
                    card.power.value = opp_card.power.base;
                    opp_card.power.value = card.power.base;
                }
            }
            Exchange::Damage => {
                if !card.damage.attr.is_blocked() {
                    card.damage.value = opp_card.damage.base;
                    opp_card.damage.value = card.damage.base;
                }
            }
            Exchange::ImposePower => {
                if !card.power.attr.is_blocked() {
                    opp_card.power.value = card.power.base;
                }
            }
            Exchange::ImposeDamage => {
                if !card.damage.attr.is_blocked() {
                    opp_card.damage.value = card.damage.base;
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct ProtectionModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    prot: Protect,
    both: bool,
}

impl ProtectionModifier {
    pub fn apply(&mut self, data: &BattleData) {
        let mut card = data.card.borrow_mut();
        if self.both {
            let mut opp_card = data.opp_card.borrow_mut();
            match self.prot {
                Protect::Power => {
                    card.power.attr.protect();
                    opp_card.power.attr.protect();
                }
                Protect::Damage => {
                    card.damage.attr.protect();
                    opp_card.damage.attr.protect();
                }
                Protect::Attack => {
                    card.attack.attr.protect();
                    opp_card.attack.attr.protect();
                }
                Protect::Ability => {
                    // card.ability.attr.protect();
                    // opp_card.ability.attr.protect();
                    card.ability.protect();
                    opp_card.ability.protect();
                }
                Protect::Bonus => {
                    // card.bonus.attr.protect();
                    // opp_card.bonus.attr.protect();
                    card.bonus.protect();
                    opp_card.bonus.protect();
                }
            };
        } else {
            match self.prot {
                Protect::Power => card.power.attr.protect(),
                Protect::Damage => card.damage.attr.protect(),
                Protect::Attack => card.attack.attr.protect(),
                // Protect::Ability => card.ability.attr.protect(),
                // Protect::Bonus => card.bonus.attr.protect(),
                Protect::Ability => card.ability.protect(),
                Protect::Bonus => card.bonus.protect(),
            };
            println!("{:?}", card.power.attr);
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct RecoverModifier {
    #[serde(rename = "eventTime")]
    event_time: EventTime,
    win: Option<bool>,
    // rec: Recover,
    n: u8,
    #[serde(rename = "outOf")]
    out_of: u8,
}

impl RecoverModifier {
    fn apply(&mut self, data: &BattleData) {
        if !data.card.borrow().pillz.is_blocked() {
            let gain = data.player_pillz_used * self.n / self.out_of;
            println!(
                "Player recovered {} pillz / {}",
                gain, data.player_pillz_used
            );
            data.player.borrow_mut().pillz += gain;
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Modifier {
    Basic(BasicModifier),
    Cancel(CancelModifier),
    Copy(CopyModifier),
    Exchange(ExchangeModifier),
    Protection(ProtectionModifier),
    Recover(RecoverModifier),
}

impl Modifier {
    #[inline]
    pub fn event_time(&self) -> EventTime {
        match self {
            Modifier::Basic(inner) => inner.event_time,
            Modifier::Cancel(inner) => inner.event_time,
            Modifier::Copy(inner) => inner.event_time,
            Modifier::Exchange(inner) => inner.event_time,
            Modifier::Protection(inner) => inner.event_time,
            Modifier::Recover(inner) => inner.event_time,
        }
    }

    #[inline]
    pub fn apply(&mut self, data: &BattleData) -> Option<Ability> {
        match self {
            Modifier::Basic(inner) => inner.apply(data),
            Modifier::Cancel(inner) => inner.apply(data),
            Modifier::Copy(inner) => return inner.apply(data),
            Modifier::Exchange(inner) => inner.apply(data),
            Modifier::Protection(inner) => inner.apply(data),
            Modifier::Recover(inner) => inner.apply(data),
        };
        None
    }
}
