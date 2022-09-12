use std::cell::RefCell;

use colored::Colorize;

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

#[derive(Debug)]
pub struct Events {
    pub events: Option<[Vec<Ability>; 10]>,
    pub global: Option<[Option<Vec<Ability>>; 10]>,
    global_count: usize,
}

impl Clone for Events {
    fn clone(&self) -> Self {
        Events {
            events: None,
            global: Clone::clone(&self.global),
            global_count: self.global_count,
        }
    }
}

impl Default for Events {
    fn default() -> Self {
        Events {
            events: None,
            global: None,
            global_count: 0,
        }
    }
}

impl Events {
    #[inline]
    pub fn init(&mut self) {
        self.events = Some(Default::default());
    }

    pub fn add(&mut self, ability: Ability) {
        if ability.modifiers.len() == 0 {
            println!("{}: {:#?}", "Failed to add ability".red(), ability);
        } else {
            match ability.ability_type.clone() {
                AbilityType::Ability | AbilityType::Bonus => {
                    self.events.as_mut().unwrap()[ability.event_time() as usize].push(ability)
                }
                // AbilityType::Global |
                AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    if self.global == None {
                        self.global = Some(Default::default())
                    }
                    let global = self.global.as_mut().unwrap();
                    let index = ability.event_time() as usize;
                    if global[index] == None {
                        global[index] = Some(Vec::new());
                    }
                    global[index].as_mut().unwrap().push(ability);
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
        if global[index] == None {
            global[index] = Some(Vec::new());
        }
        global[index].as_mut().unwrap().push(ability);
        self.global_count += 1;
    }

    pub fn execute(&mut self, event: EventTime, data: &BattleData) {
        let index = event as usize;
        let events = &mut self.events.as_mut().unwrap()[index];

        let mut new_abilities = Vec::<Ability>::with_capacity(2);
        for ability in events.iter_mut() {
            if let Some(new_ability) = ability.apply(data) {
                new_abilities.push(new_ability);
            }
        }
        // events.clear();

        if self.global_count != 0 {
            let mut clear = false;
            if let Some(events) = self.global.as_mut().unwrap()[index].as_mut() {
                for ability in events.iter_mut() {
                    ability.apply(data);
                }

                let len = events.len();
                events.retain(|ab| ab.remove == false);
                self.global_count -= len - events.len();
                if events.len() == 0 {
                    clear = true;
                }
            }
            if clear {
                if self.global_count == 0 {
                    self.global = None;
                } else {
                    self.global.as_mut().unwrap()[index] = None;
                }
            }
        }

        for new_ability in new_abilities {
            self.add(new_ability);
        }
    }

    pub fn check_cancels(&mut self, data: &BattleData) -> bool {
        let mut changed = false;
        for ability in self.events.as_mut().unwrap()[EventTime::PRE4 as usize].iter_mut() {
            // let ability_type = ability.ability_type.clone();
            if let Modifier::Cancel(modifier) = &mut ability.modifiers[0] {
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
        self.events = None;
    }
}

pub struct BattleData<'a> {
    pub round: u8,
    pub first: bool,
    pub player: &'a RefCell<Player>,
    pub hand: &'a Hand,
    pub card: &'a RefCell<Card>,
    pub player_pillz_used: u8,
    pub opp: &'a RefCell<Player>,
    pub opp_hand: &'a Hand,
    pub opp_card: &'a RefCell<Card>,
    pub opp_pillz_used: u8,
    pub events: &'a RefCell<Events>,
}
