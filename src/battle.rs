use std::cell::RefCell;

use colored::Colorize;

use crate::{
    ability::{Ability, AbilityType},
    card::{Card, HandCell},
    game::Player,
    modifiers::{EventTime, Modifier},
    utils::StackVec4,
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

#[derive(Debug, Clone, Copy)]
pub struct Events {
    events: StackVec4<(EventTime, Ability)>,
    global: Option<StackVec4<(EventTime, Ability)>>,
    // events: [(EventTime, Ability); 4],
    // global: Option<[(EventTime, Ability); 4]>,
}

impl Default for Events {
    #[inline]
    fn default() -> Self {
        Events {
            events: Default::default(),
            global: None,
        }
    }
}

impl Events {
    pub fn add(&mut self, ability: Ability) {
        if ability.modifiers.len() == 0 {
            println!("{}: {:#?}", "Failed to add ability".red(), ability);
        } else {
            match ability.ability_type {
                AbilityType::Ability | AbilityType::Bonus => {
                    self.events.push((ability.event_time(), ability));
                }
                AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    if self.global == None {
                        self.global = Some(StackVec4 {
                            len: 1,
                            data: [Some((ability.event_time(), ability)), None, None, None],
                        });
                    } else {
                        self.global.unwrap().push((ability.event_time(), ability));
                    }
                }
                _ => (),
            }
        }
    }

    pub fn add_global(&mut self, ability: Ability) {
        if ability.modifiers.len() == 0 {
            println!("{}: {:#?}", "Failed to add global ability".red(), ability);
            return;
        }

        if self.global == None {
            self.global = Some(Default::default());
        }
        self.global.unwrap().push((ability.event_time(), ability));
    }

    pub fn execute(&mut self, event: EventTime, data: &BattleData) {
        if self.events.len != 0 {
            let mut new_abilities = Vec::<Ability>::new();
            for item in self.events.data.iter_mut() {
                if let Some((et, ability)) = item && event == *et {
                    if let Some(new_ability) = ability.apply(data) {
                        new_abilities.push(new_ability);
                    }
                }
            }
            for ability in new_abilities {
                self.add(ability);
            }
        }

        if let Some(global) = self.global.as_mut() {
            for item in global.data.iter_mut() {
                if let Some((et, ability)) = item && event == *et {
                    ability.apply(data);
                    if ability.remove {
                        *item = None;
                    }
                }
            }
        }
    }

    pub fn check_cancels(&mut self, data: &BattleData) -> bool {
        let mut changed = false;
        for item in self.events.data.iter_mut() {
            if let Some((et, ability)) = item && *et == EventTime::PRE4 {
                if let Some(Modifier::Cancel(mut modifier)) = ability.modifiers[0].clone() {
                    if modifier.applied == None {
                        continue;
                    }

                    let applied = modifier.applied.unwrap();

                    // println!("Applied is some: {:?}", ability.ability_type);
                    if ability.ability_type == AbilityType::Ability {
                        println!("{:?}", data.card.borrow());
                        // if data.card.borrow().ability.attr.is_blocked() == applied {
                        if data.card.borrow().ability.is_blocked() == applied {
                            println!("{}: {:?}", "Undoing ability".red(), modifier);
                            if applied {
                                println!("{}: {:?}", "Undoing bonus".red(), modifier);
                                modifier.undo(data);
                            } else {
                                println!("{}: {:?}", "Redoing bonus".yellow(), modifier);
                                modifier.apply(data);
                            }
                            changed = true;
                        }
                    } else if ability.ability_type == AbilityType::Bonus {
                        // println!("{:?}", data.card.borrow());
                        // if data.card.borrow().bonus.attr.is_blocked() == applied {
                        if data.card.borrow().bonus.is_blocked() == applied {
                            if applied {
                                println!("{}: {:?}", "Undoing bonus".red(), modifier);
                                modifier.undo(data);
                            } else {
                                println!("{}: {:?}", "Redoing bonus".yellow(), modifier);
                                modifier.apply(data);
                            }
                            changed = true;
                        }
                    }
                }
            }
        }
        changed
    }

    #[inline]
    pub fn execute_start(&mut self, data: &BattleData) {
        self.execute(EventTime::START, data);
    }

    // pub fn execute_pre(&mut self, data: &BattleData) {
    //     for e in [
    //         EventTime::PRE4,
    //         EventTime::PRE3,
    //         EventTime::PRE2,
    //         EventTime::PRE1,
    //     ] {
    //         self.execute(e, data);
    //     }
    // }

    #[inline]
    pub fn execute_post(&mut self, data: &BattleData) {
        for e in [
            EventTime::POST1,
            EventTime::POST2,
            EventTime::POST3,
            EventTime::POST4,
        ] {
            self.execute(e, data);
        }
    }

    #[inline]
    pub fn execute_end(&mut self, data: &BattleData) {
        self.execute(EventTime::END, data);
        self.events.len = 0;
        self.events.data = [None, None, None, None];
    }
}

pub struct BattleData<'a> {
    pub round: u8,
    pub first: bool,
    pub player: &'a RefCell<&'a mut Player>,
    pub hand: &'a HandCell<'a>,
    pub card: &'a RefCell<&'a mut Card>,
    pub player_pillz_used: u8,
    pub opp: &'a RefCell<&'a mut Player>,
    pub opp_hand: &'a HandCell<'a>,
    pub opp_card: &'a RefCell<&'a mut Card>,
    pub opp_pillz_used: u8,
    pub events: &'a RefCell<&'a mut Events>,
}
