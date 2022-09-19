use std::{cell::RefCell, mem::swap};

use colored::Colorize;
use tinyvec::ArrayVec;

use crate::{
    ability::{Ability, AbilityType},
    card::{Card, HandCell},
    game::Player,
    modifiers::{EventTime, Modifier},
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
    pub events: [ArrayVec<[Option<Ability>; 4]>; 10],
    pub global: Option<[ArrayVec<[Option<Ability>; 4]>; 10]>,
    global_count: usize,
}

impl Default for Events {
    #[inline]
    fn default() -> Self {
        Events {
            events: Default::default(),
            global: None,
            global_count: 0,
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
                    self.events[ability.event_time() as usize].push(Some(ability))
                }
                // AbilityType::Global |
                AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    if self.global == None {
                        self.global = Some(Default::default())
                    }
                    let global = self.global.as_mut().unwrap();
                    let index = ability.event_time() as usize;
                    global[index].push(Some(ability));
                    self.global_count += 1;
                }
                _ => (),
            }
        }
    }

    pub fn add_global(&mut self, ability: Ability) {
        if self.global == None {
            self.global = Some(Default::default());
        }
        let global = self.global.as_mut().unwrap();
        let index = ability.event_time() as usize;
        global[index].push(Some(ability));
        self.global_count += 1;
    }

    pub fn execute(&mut self, event: EventTime, data: &BattleData) {
        let index = event as usize;

        let x = &mut self.events[index];
        if x.len() != 0 {
            let mut events = Default::default();
            swap(x, &mut events);

            for ability in events.iter_mut() {
                if let Some(new_ability) = ability.as_mut().unwrap().apply(data) {
                    self.add(new_ability);
                }
            }
        }

        if self.global_count != 0 {
            let events = &mut self.global.as_mut().unwrap()[index];
            for ability in events.iter_mut() {
                ability.as_mut().unwrap().apply(data);
            }

            let len = events.len();
            events.retain(|ab| !ab.as_ref().unwrap().remove);
            self.global_count -= len - events.len();
        }
    }

    pub fn check_cancels(&mut self, data: &BattleData) -> bool {
        let mut changed = false;
        for ability in self.events[EventTime::PRE4 as usize].iter_mut() {
            // let ability_type = ability.ability_type.clone();
            let ability = ability.as_mut().unwrap();
            if let Some(Modifier::Cancel(modifier)) = &mut ability.modifiers[0] {
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
        // self.events = None;
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
