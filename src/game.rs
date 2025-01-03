use std::{
    cell::RefCell,
    fmt::Display,
    hash::Hash,
    sync::atomic::{AtomicU32, Ordering},
};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{
    ability::AbilityType,
    battle::{BattleData, Events},
    card::Hand,
    modifiers::EventTime,
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

pub static mut BATTLE_COUNT: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoundWin {
    WIN,
    LOSE,
    NONE,
}

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub player_type: PlayerType,
    pub life: u8,
    pub life_previous: u8,
    pub pillz: u8,
    pub pillz_previous: u8,
    pub won: RoundWin,
    pub won_previous: RoundWin,
}

impl Player {
    pub fn new(player_type: PlayerType) -> Self {
        Player {
            player_type,
            life: 12,
            life_previous: 12,
            pillz: 12,
            pillz_previous: 12,
            won: RoundWin::NONE,
            won_previous: RoundWin::NONE,
        }
    }
    fn name(&self) -> ColoredString {
        if self.player_type == PlayerType::Player {
            " Player ".bright_white().on_cyan()
        } else {
            " Opponent ".bright_white().on_red()
        }
    }
    pub fn print(&self) {
        print!(" {:^10}  ", self.name());

        let life_change = 12.min(self.life_previous) as i8 - self.life as i8;
        let life_lost = if life_change > 0 {
            life_change as usize
        } else {
            0
        };
        let life_empty = if self.life < 12 {
            12 - 12.min(self.life) as usize - life_lost
        } else {
            0
        };
        print!(
            "{} | {:<2} ",
            "Life".bright_red(),
            self.life.to_string().red()
        );
        print!(
            "{}  ",
            format!(
                "{:<w1$}{}{}",
                "".black().on_bright_red(),
                // self.pillz.to_string().black().on_bright_red(),
                " ".repeat(life_lost).on_white(),
                " ".repeat(life_empty).on_bright_black(),
                w1 = 12.min(self.life as usize)
            )
            .on_white()
        );

        let pillz_change = 12.min(self.pillz_previous) as i8 - self.pillz as i8;
        let pillz_lost = if pillz_change > 0 {
            pillz_change as usize
        } else {
            0
        };
        let pillz_empty = if self.life < 12 {
            12 - 12.min(self.pillz) as usize - pillz_lost
        } else {
            0
        };
        print!(
            "{} | {:<2} ",
            "Pillz".bright_blue(),
            self.pillz.to_string().blue(),
            // format!("{} {}",
            //     self.pillz.to_string().blue(),
            //     (-(pillz_used as i8)).to_string().bright_black()
            // )
        );
        // print!("{:<3} ", (-(pillz_used as i32)).to_string().bright_black());
        println!(
            "{}",
            format!(
                "{:<w1$}{}{}",
                "".black().on_bright_blue(),
                // self.pillz.to_string().black().on_bright_blue(),
                " ".repeat(pillz_lost).on_white(),
                " ".repeat(pillz_empty).on_bright_black(),
                w1 = self.pillz as usize
            )
            .on_white()
        );
    }
}

#[derive(Clone, Copy, Default, Debug, Eq, PartialEq, Deserialize, Hash)]
pub struct Selection {
    pub index: usize,
    #[serde(default)]
    pub pillz: u8,
    #[serde(default)]
    pub fury: bool,
}

impl Display for Selection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.fury {
            write!(
                f,
                "{} {} {}",
                self.index,
                self.pillz,
                format!("{}", self.fury).red()
            )?;
        } else {
            write!(f, "{} {} {}", self.index, self.pillz, self.fury)?;
        }

        Ok(())
    }
}

impl Selection {
    pub fn new(index: usize, pillz: u8, fury: bool) -> Self {
        Selection { index, pillz, fury }
    }
    pub fn parse(input: String) -> Option<Selection> {
        let tokens = input.split(" ");

        let mut index = 0usize;
        let mut pillz = 0u8;
        let mut fury = false;

        for (i, token) in tokens.enumerate() {
            if i == 0 {
                match token.parse::<usize>() {
                    Err(_) => return None,
                    Ok(val) => index = val,
                };
            } else if i == 1 {
                match token.parse::<u8>() {
                    Err(_) => return None,
                    Ok(val) => pillz = val,
                };
            } else {
                match token.parse::<bool>() {
                    Err(_) => return None,
                    Ok(val) => fury = val,
                };
                break;
            }
        }

        Some(Selection { index, pillz, fury })
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameStatus {
    Player,
    Opponent,
    Draw,
    Playing,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayerType {
    Player,
    Opponent,
}

#[derive(Debug, Clone, Copy)]
pub struct Game {
    pub round: u8,
    pub p1: Player,
    pub p2: Player,
    pub h1: Hand,
    pub h2: Hand,
    pub s1: Option<Selection>,
    pub s2: Option<Selection>,
    pub events1: Events,
    pub events2: Events,
    pub flip: u8,
}

#[allow(dead_code)]
impl Game {
    pub fn new(h1: Hand, h2: Hand) -> Self {
        // let h1 = Hand::from_ids(1182, 271, 1300, 1906);
        // let h1 = Hand::from_ids(1182, 271, 271, 1906);
        // let h1 = Hand::random_hand_clan(Clan::UluWatu);
        // let h2 = Hand::from_ids(2118, 2157, 488, 1019);
        // let h2 = Hand::from_ids(2118, 2133, 2133, 2133);

        let mut events1 = Events::default();
        let mut events2 = Events::default();

        if let Some(leader1) = h1.get_leader() {
            events1.add_global(leader1.get_ability());
        }
        if let Some(leader2) = h2.get_leader() {
            events2.add_global(leader2.get_ability());
        }

        Game {
            round: 0,
            // p1: RefCell::new(Player::new("Player", PlayerType::Player)),
            // p2: RefCell::new(Player::new("Opponent", PlayerType::Opponent)),
            p1: Player::new(PlayerType::Player),
            p2: Player::new(PlayerType::Opponent),
            h1,
            h2,
            s1: None,
            s2: None,
            events1: events1,
            events2: events2,
            flip: 0,
        }
    }
    pub fn random() -> Self {
        let h1 = Hand::random_hand_clan(Clan::GHEIST);
        let h2 = Hand::random_hand_clan(Clan::Nightmare);
        Game::new(h1, h2)
    }
    pub fn has_global(&self) -> bool {
        for card in self.h1.cards.iter() {
            match card.get_ability().ability_type {
                AbilityType::Global | AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    return true;
                }
                _ => (),
            }
            match card.get_bonus().ability_type {
                AbilityType::Global | AbilityType::GlobalAbility | AbilityType::GlobalBonus => {
                    return true;
                }
                _ => (),
            }
        }
        false
    }
    pub fn status(&self) -> GameStatus {
        if self.s1.is_none() != self.s2.is_none() {
            return GameStatus::Playing;
        }

        let l1 = self.p1.life;
        let l2 = self.p2.life;

        if l1 <= 0 && l2 <= 0 {
            GameStatus::Draw
        } else if l1 <= 0 {
            GameStatus::Opponent
        } else if l2 <= 0 {
            GameStatus::Player
        } else if self.round == 4 {
            if l1 == l2 {
                GameStatus::Draw
            } else if l1 > l2 {
                GameStatus::Player
            } else {
                GameStatus::Opponent
            }
        } else {
            GameStatus::Playing
        }
    }

    pub fn print_status(&self) {
        unsafe {
            if !PRINT {
                return;
            }
        }

        match self.status() {
            GameStatus::Playing => {
                self.p2.print();
                self.h2.print(
                    self.s2
                        .unwrap_or(Selection {
                            index: 4,
                            pillz: 0,
                            fury: false,
                        })
                        .index,
                );
                self.p1.print();
                self.h1.print(
                    self.s1
                        .unwrap_or(Selection {
                            index: 4,
                            pillz: 0,
                            fury: false,
                        })
                        .index,
                );
            }
            GameStatus::Player => println!(
                "\n{} is the Winner!\n",
                format!(" {} ", "Player").bright_white().on_bright_blue()
            ),
            GameStatus::Opponent => println!(
                "\n{} is the Winner!\n",
                format!(" {} ", "Opponent").bright_white().on_bright_red()
            ),
            GameStatus::Draw => println!(
                "\n{} Game ends in a Draw\n",
                format!(" {} ", "Game Over!").black().on_red()
            ),
        }
    }

    #[inline]
    pub fn get_turn(&self) -> PlayerType {
        if self.round % 2 == self.flip {
            if self.s1.is_some() {
                PlayerType::Opponent
            } else {
                PlayerType::Player
            }
        } else if self.s2.is_some() {
            PlayerType::Player
        } else {
            PlayerType::Opponent
        }
    }
    pub fn get_turn_name(&self) -> ColoredString {
        if self.get_turn() == PlayerType::Player {
            " Player ".white().on_cyan()
        } else {
            " Opponent ".white().on_red()
        }
    }
    pub fn get_turn_player(&self) -> &Player {
        if self.get_turn() == PlayerType::Player {
            &self.p1
        } else {
            &self.p2
        }
    }
    pub fn get_turn_opponent(&self) -> &Player {
        if self.get_turn() == PlayerType::Player {
            &self.p2
        } else {
            &self.p1
        }
    }
    pub fn get_turn_hand(&self) -> &Hand {
        if self.get_turn() == PlayerType::Player {
            &self.h1
        } else {
            &self.h2
        }
    }
    pub fn get_turn_opponent_hand(&self) -> &Hand {
        if self.get_turn() == PlayerType::Player {
            &self.h2
        } else {
            &self.h1
        }
    }
    pub fn get_first_turn(&self) -> PlayerType {
        if self.round % 2 == self.flip {
            PlayerType::Player
        } else {
            PlayerType::Opponent
        }
    }
    pub fn has_someone_selected(&self) -> bool {
        self.s1.is_some() || self.s2.is_some()
    }

    fn print_battle(&self, attack1: u8, attack2: u8) {
        unsafe {
            if !PRINT {
                return;
            }
        }

        match (self.s1, self.s2) {
            (Some(s1), Some(s2)) => {
                // if let Some(s1) = self.s1 && let Some(s2) = self.s2 {
                self.h2.print(s2.index);
                self.p2.print();

                print!("\n{}", " ".repeat(32));
                if self.p2.won == RoundWin::WIN {
                    print!("{}  ", " Winner! ".black().on_magenta());
                } else {
                    print!("{}", " ".repeat(11));
                }
                print!(
                    "{}{}  ",
                    " Pillz ".bright_white().on_red(),
                    format!(" {} ", s2.pillz).red().on_white()
                );
                print!(
                    "{}{}  ",
                    " Attack ".bright_red().on_white(),
                    format!(" {} ", attack2).black().on_red()
                );
                if s2.fury {
                    print!("{} ", " FURY! ".bright_red().on_black());
                }
                println!();

                print!("\n{}", " ".repeat(51));
                println!("{}", " VERSUS ".black().on_bright_green());

                print!("\n{}", " ".repeat(32));
                if self.p1.won == RoundWin::WIN {
                    print!("{}  ", " Winner! ".black().on_magenta());
                } else {
                    print!("{}", " ".repeat(11));
                }
                print!(
                    "{}{}  ",
                    " Pillz ".bright_white().on_blue(),
                    format!(" {} ", s1.pillz).blue().on_white()
                );
                print!(
                    "{}{}  ",
                    " Attack ".bright_blue().on_white(),
                    format!(" {} ", attack1).black().on_blue()
                );
                if s1.fury {
                    print!("{} ", " FURY! ".bright_blue().on_black());
                }
                println!("\n");

                self.p1.print();
                self.h1.print(s1.index);
            }
            _ => unreachable!(),
        }
    }

    fn battle(&mut self) {
        let s1 = &mut self.s1.unwrap();
        let s2 = &mut self.s2.unwrap();

        let first_turn = self.get_first_turn();
        let pillz1 = s1.pillz;
        let pillz2 = s2.pillz;
        let fury1 = s1.fury;
        let fury2 = s2.fury;

        let total_pillz1 = if fury1 { pillz1 + 3 } else { pillz1 };
        let total_pillz2 = if fury2 { pillz2 + 3 } else { pillz2 };

        assert!(
            total_pillz1 <= self.p1.pillz,
            "{}, {}",
            total_pillz1,
            self.p1.pillz
        );
        assert!(
            total_pillz2 <= self.p2.pillz,
            "{}, {}",
            total_pillz2,
            self.p2.pillz
        );

        self.p1.won_previous = self.p1.won;
        self.p2.won_previous = self.p2.won;
        self.p1.life_previous = self.p1.life;
        self.p2.life_previous = self.p2.life;
        self.p1.pillz_previous = self.p1.pillz;
        self.p2.pillz_previous = self.p2.pillz;

        {
            let card1 = &self.h1[s1.index];
            let card2 = &self.h2[s2.index];

            self.events1.add(card1.get_ability());
            self.events2.add(card2.get_ability());
            self.events1.add(card1.get_bonus());
            self.events2.add(card2.get_bonus());
        }

        let Game {
            events1,
            events2,
            h1,
            h2,
            p1,
            p2,
            ..
        } = self;

        let events1 = RefCell::new(events1);
        let events2 = RefCell::new(events2);

        let p1 = RefCell::new(p1);
        let p2 = RefCell::new(p2);

        let h1 = h1.to_handcell();
        let h2 = h2.to_handcell();

        let card1 = &h1.cards[s1.index];
        let card2 = &h2.cards[s2.index];

        let battle_data1 = BattleData {
            round: self.round,
            first: first_turn == PlayerType::Player,
            hand: &h1,
            opp_hand: &h2,
            player: &p1,
            card: card1,
            player_pillz_used: total_pillz1,
            opp: &p2,
            opp_card: card2,
            opp_pillz_used: total_pillz2,
            events: &events1,
        };
        let battle_data2 = BattleData {
            round: self.round,
            first: first_turn == PlayerType::Opponent,
            hand: &h2,
            opp_hand: &h1,
            player: &p2,
            card: &card2,
            player_pillz_used: total_pillz2,
            opp: &p1,
            opp_card: &card1,
            opp_pillz_used: total_pillz1,
            events: &events2,
        };

        {
            let mut events1 = events1.borrow_mut();
            let mut events2 = events2.borrow_mut();

            events1.execute_start(&battle_data1);
            events2.execute_start(&battle_data2);

            // events1.execute_pre(&battle_data1);
            // events2.execute_pre(&battle_data2);
            events1.execute(EventTime::PRE4, &battle_data1);
            events2.execute(EventTime::PRE4, &battle_data2);
            for _ in 0..3 {
                let changed1 = events1.check_cancels(&battle_data1);
                let changed2 = events2.check_cancels(&battle_data2);
                if !changed1 && !changed2 {
                    break;
                }
            }
            events1.execute(EventTime::PRE3, &battle_data1);
            events2.execute(EventTime::PRE3, &battle_data2);
            events1.execute(EventTime::PRE2, &battle_data1);
            events2.execute(EventTime::PRE2, &battle_data2);
            events1.execute(EventTime::PRE1, &battle_data1);
            events2.execute(EventTime::PRE1, &battle_data2);
        }

        if fury1 {
            card1.borrow_mut().damage.value += 2;
        }
        if fury2 {
            card2.borrow_mut().damage.value += 2;
        }

        let attack1 = (pillz1 + 1) * card1.borrow().power.value;
        let attack2 = (pillz2 + 1) * card2.borrow().power.value;

        card1.borrow_mut().attack.value = attack1;
        card2.borrow_mut().attack.value = attack2;

        events1.borrow_mut().execute_post(&battle_data1);
        events2.borrow_mut().execute_post(&battle_data2);

        let attack1 = card1.borrow().attack.value;
        let attack2 = card2.borrow().attack.value;
        {
            let mut p1 = p1.borrow_mut();
            let mut p2 = p2.borrow_mut();
            let mut card1 = card1.borrow_mut();
            let mut card2 = card2.borrow_mut();
            if attack1 > attack2
                || (attack1 == attack2
                    && (card1.level < card2.level
                        || (card1.level == card2.level && first_turn == PlayerType::Player)))
            {
                p2.life -= card1.damage.value.min(p2.life);
                card1.won = true;
                p1.won = RoundWin::WIN;
                p2.won = RoundWin::LOSE;
            } else {
                p1.life -= card2.damage.value.min(p1.life);
                card2.won = true;
                p2.won = RoundWin::WIN;
                p1.won = RoundWin::LOSE;
            }

            p1.pillz -= total_pillz1;
            p2.pillz -= total_pillz2;
        }

        events1.borrow_mut().execute_end(&battle_data1);
        events2.borrow_mut().execute_end(&battle_data2);

        card1.borrow_mut().played = true;
        card2.borrow_mut().played = true;

        self.print_battle(attack1, attack2);

        self.round += 1;

        unsafe {
            // *BATTLE_COUNT.get_mut() += 1;
            BATTLE_COUNT.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn can_select(&self, index: usize, pillz: u8, fury: bool) -> bool {
        if index > 3 {
            return false;
        }
        let p = self.get_turn_player().pillz;
        let pillz = if fury { pillz + 3 } else { pillz };
        if pillz > p || self.get_turn_hand()[index].played {
            return false;
        }
        true
    }

    pub fn select(&mut self, index: usize, pillz: u8, fury: bool) -> bool {
        let s = Some(Selection { index, pillz, fury });
        if self.round % 2 == self.flip {
            if self.s1.is_some() {
                self.s2 = s;
                self.battle();
                self.s1 = None;
                self.s2 = None;
                true
            } else {
                self.s1 = s;
                self.print_status();
                false
            }
        } else if self.s2.is_some() {
            self.s1 = s;
            self.battle();
            self.s1 = None;
            self.s2 = None;
            true
        } else {
            self.s2 = s;
            self.print_status();
            false
        }
    }

    pub fn select_both(&mut self, s1: Selection, s2: Selection) {
        self.s1 = Some(s1);
        self.s2 = Some(s2);
        self.battle();
        self.s1 = None;
        self.s2 = None;
    }

    pub fn clear_selection(&mut self) {
        self.s1 = None;
        self.s2 = None;
    }
}
