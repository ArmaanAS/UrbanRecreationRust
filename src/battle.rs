use colored::Colorize;
use std::cell::RefCell;

use crate::{
    ability::{Ability, AbilityType},
    card::{Card, Hand},
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

#[derive(Clone, Debug)]
pub struct Events {
    pub events: [Vec<Ability>; 10],
    pub global: [Vec<Ability>; 10],
}

impl Default for Events {
    fn default() -> Self {
        Events {
            events: Default::default(),
            global: Default::default(),
        }
    }
}

impl Events {
    pub fn add(&mut self, ability: Ability) {
        if ability.modifiers.len() == 0 {
            println!("{}: {:#?}", "Failed to add ability".red(), ability);
        } else {
            match ability.ability_type.clone() {
                AbilityType::Ability | AbilityType::Bonus => {
                    self.events[ability.event_time() as usize].push(ability)
                }
                // AbilityType::Global |
                AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    self.global[ability.event_time() as usize].push(ability)
                }
                _ => (),
            }
        }
    }

    pub fn add_global(&mut self, ability: Ability) {
        self.global[ability.event_time() as usize].push(ability);
    }

    pub fn execute(&mut self, event: EventTime, data: &BattleData) {
        let events = &mut self.events[event as usize];

        let mut new_abilities = Vec::<Ability>::with_capacity(2);
        for ability in events.iter_mut() {
            if let Some(new_ability) = ability.apply(data) {
                new_abilities.push(new_ability);
            }
        }
        // events.clear();

        for ability in self.global[event as usize].iter_mut() {
            ability.apply(data);
        }
        for events in self.global.iter_mut() {
            // if events.len() != 0 {
            // println!("Before: {}", events.len());
            events.retain(|ab| ab.remove == None);
            // println!("After: {}", events.len());
            // }
        }

        for new_ability in new_abilities {
            self.add(new_ability);
        }
    }

    pub fn check_cancels(&mut self, data: &BattleData) -> bool {
        let mut changed = false;
        for ability in self.events[EventTime::PRE4 as usize].iter_mut() {
            // let ability_type = ability.ability_type.clone();
            if let Modifier::Cancel(modifier) = &mut ability.modifiers[0] {
                if modifier.applied == None {
                    continue;
                }

                let applied = modifier.applied.unwrap();

                // println!("Applied is some: {:?}", ability.ability_type);
                if ability.ability_type == AbilityType::Ability {
                    println!("{:?}", data.card.borrow());
                    if data.card.borrow().ability.attr.is_blocked() == applied {
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
                    if data.card.borrow().bonus.attr.is_blocked() == applied {
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

    pub fn execute_end(&mut self, data: &BattleData) {
        self.execute(EventTime::END, data);
        for events in self.events.iter_mut() {
            events.clear();
        }
    }
}

pub struct PlayerRound<'a> {
    pub round: u8,
    pub first: bool,
    pub hand: &'a Hand,
    pub opp_hand: &'a Hand,
}

// impl PlayerRound<'_> {
//     fn next(&mut self, first: bool) {
//         self.round += 1;

//         self.first = first;
//     }
// }

pub struct BattleData<'a> {
    pub round: PlayerRound<'a>,
    pub player: &'a RefCell<Player>,
    pub card: &'a RefCell<Card>,
    pub player_pillz_used: u8,
    pub opp: &'a RefCell<Player>,
    pub opp_card: &'a RefCell<Card>,
    pub opp_pillz_used: u8,
    pub events: &'a RefCell<Events>,
}
