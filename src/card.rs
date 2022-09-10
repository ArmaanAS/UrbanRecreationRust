use chrono::{Datelike, NaiveDateTime};
use colored::{Color, Colorize};
use lazy_static::lazy_static;
use rand::{seq::SliceRandom, thread_rng};
use regex::Captures;
use serde::Deserialize;
use simd_json::from_reader;
use std::{
    cell::{Ref, RefCell},
    collections::{HashMap, HashSet},
    fmt::{Display, Formatter, Result},
    fs::File,
    path::Path,
};

use crate::{
    ability::{Ability, ABILITIES, CLANS_REGEX},
    types::{Clan, Rarity},
};

#[derive(Clone, Debug, Deserialize)]
pub struct CardData {
    pub id: u32,
    pub name: String,
    #[serde(rename = "clan_id")]
    pub clan: Clan,
    pub level: u8,
    pub level_max: u8,
    pub power: u8,
    pub damage: u8,
    pub rarity: String,
    pub ability_id: u32,
    pub ability: String,
    pub bonus_id: u32,
    pub bonus: String,
    pub release_date: u32,
}

#[derive(Clone, Debug)]
pub struct CardAttr {
    pub cancelled: u8,
    protected: u8,
}

impl Default for CardAttr {
    fn default() -> Self {
        CardAttr {
            cancelled: 0,
            protected: 0,
        }
    }
}

impl CardAttr {
    #[inline]
    pub fn cancel(&mut self) {
        self.cancelled += 1;
    }
    #[inline]
    pub fn remove_cancel(&mut self) {
        self.cancelled -= 1;
    }
    #[inline]
    pub fn protect(&mut self) {
        self.protected += 1;
    }
    // #[inline]
    // pub fn remove_protect(&mut self) {
    //     self.protected -= 1;
    // }
    #[inline]
    pub fn is_blocked(&self) -> bool {
        // self.protect || !self.cancel
        self.protected == 0 && self.cancelled != 0
    }
    #[inline]
    pub fn is_protected(&self) -> bool {
        // self.protect || !self.cancel
        self.protected != 0
    }
}

#[derive(Clone, Debug)]
pub struct CardStat {
    pub attr: CardAttr,
    pub base: u8,
    pub value: u8,
}

#[derive(Clone, Debug)]
pub enum AbilityString {
    None,
    String(String),
}

#[derive(Clone, Debug)]
pub struct CardAbility {
    pub attr: CardAttr,
    pub string: AbilityString,
}

impl CardStat {
    fn new(val: u8) -> Self {
        CardStat {
            attr: CardAttr::default(),
            base: val,
            value: val,
        }
    }
}

impl Default for CardStat {
    fn default() -> Self {
        CardStat {
            attr: CardAttr::default(),
            base: 0,
            value: 0,
        }
    }
}

lazy_static! {
    static ref CARDS: Vec<CardData> = {
        let data_file =
            File::open(Path::new("./assets/data.json")).expect("file should open read only");
        let mut cards: Vec<CardData> =
            from_reader(data_file).expect("Error while reading JSON file");

        for card in cards.iter_mut() {
            let ability_str = card.ability.clone();
            let ability = CLANS_REGEX.replace_all(ability_str.as_str(), |caps: &Captures| {
                Clan::from(*&caps[1].parse::<u8>().unwrap())
                    .short_name()
                    .to_string()
                    + " "
            });
            // if ability_str != ability {
            //     println!("{}", ability);
            // }
            card.ability = ability.to_string();
        }

        cards
    };
    pub static ref CARD_IDS: HashMap<u32, CardData> = {
        let mut map = HashMap::new();
        for card in CARDS.iter() {
            map.insert(card.id, card.clone());
        }
        map
    };
    static ref CARD_NAMES: HashMap<String, CardData> = {
        let mut map = HashMap::new();
        for card in CARDS.iter() {
            map.insert(card.name.to_string().to_ascii_lowercase(), card.clone());
        }
        map
    };
    static ref CARD_CLANS: HashMap<Clan, Vec<CardData>> = {
        let mut map = HashMap::<Clan, Vec<CardData>>::new();
        for card in CARDS.iter() {
            if !map.contains_key(&card.clan) {
                map.insert(card.clan, Vec::new());
            }

            let clan_cards = map.get_mut(&card.clan).unwrap();
            clan_cards.push(card.clone());
        }
        map
    };
    // static ref CLANS_ABILITY_IDS: HashMap<Clan, u32> = {
    //     let mut map = HashMap::new();
    //     for card in CARDS.iter() {
    //         if !map.contains_key(&card.clan) {
    //             map.insert(card.clan, card.bonus_id);
    //         }
    //     }
    //     map
    // };
    // static ref CLANS_ABILITY_ATTRS: HashMap<Clan, u32> = {
    //     let mut map = HashMap::new();
    //     for card in CARDS.iter() {
    //         if !map.contains_key(&card.clan) {
    //             map.insert(card.clan, card.bonus_id);
    //         }
    //     }
    //     map
    // };
}

impl CardData {
    pub fn year(&self) -> u32 {
        NaiveDateTime::from_timestamp(self.release_date as i64, 0)
            .date()
            .year() as u32
    }
    pub fn get_id(id: u32) -> CardData {
        CARD_IDS.get(&id).unwrap().clone()
    }
    pub fn get_name(name: &str) -> CardData {
        CARD_NAMES
            .get(&name.to_string().to_ascii_lowercase())
            .unwrap()
            .clone()
    }
    pub fn to_card(&self, index: usize) -> Card {
        Card::from(self, index)
    }
}

impl CardAbility {
    fn new(s: &String) -> Self {
        CardAbility {
            attr: CardAttr::default(),
            string: match s.as_str() {
                "No ability" | "No bonus" => AbilityString::None,
                _ => {
                    let rep = CLANS_REGEX.replace_all(s.as_str(), |caps: &Captures| {
                        Clan::from(*&caps[1].parse::<u8>().unwrap())
                            .short_name()
                            .to_string()
                            + " "
                    });
                    AbilityString::String(rep.to_string())
                }
            },
        }
    }
}

impl Display for AbilityString {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if let AbilityString::String(s) = self {
            write!(f, "{}", s)
        } else {
            write!(f, "No ability")
        }
    }
}

#[derive(Clone, Debug)]
pub struct Card {
    pub played: bool,
    pub won: bool,
    pub index: usize,
    pub id: u32,
    pub name: String,
    pub clan: Clan,
    pub level: u8,
    pub level_max: u8,
    pub rarity: Rarity,
    pub year: u32,
    pub power: CardStat,
    pub damage: CardStat,
    pub ability_id: u32,
    pub ability: CardAbility,
    pub bonus_id: u32,
    pub bonus: CardAbility,
    pub attack: CardStat,
    pub life: CardAttr,
    pub pillz: CardAttr,
}

fn split_lines(s: &String, len: usize, min: usize) -> Vec<String> {
    let mut lines = Vec::<String>::with_capacity(min);

    // let re = Regex::new(r" ").unwrap();
    // let words = re.split(s.trim());
    let words = s.trim().split(" ");

    let mut line = "".to_string();
    for word in words {
        if line.len() == 0 {
            line = word.to_string() + " ";
        } else if line.len() + word.len() <= len {
            line += word;
            if line.len() < len {
                line += " ";
            }
        } else {
            line.push_str(&" ".repeat(0.max(len - line.len())));
            lines.push(line);
            line = word.to_string();
            line += " ";
        }
    }
    line.push_str(&" ".repeat(0.max(len - 20.min(line.len()))));
    lines.push(line);

    for _ in 0..min - min.min(lines.len()) {
        lines.push(" ".repeat(len).to_string());
    }

    return lines;
}

impl Card {
    pub fn from(data: &CardData, index: usize) -> Self {
        Card {
            played: false,
            won: false,
            index,
            id: data.id,
            name: data.name.to_string(),
            clan: data.clan,
            level: data.level,
            level_max: data.level_max,
            rarity: Rarity::from(&data.rarity),
            year: data.year(),
            ability_id: data.ability_id,
            ability: CardAbility::new(&data.ability),
            bonus_id: data.bonus_id,
            bonus: CardAbility::new(&data.bonus),
            power: CardStat::new(data.power),
            damage: CardStat::new(data.damage),
            attack: CardStat::default(),
            life: CardAttr::default(),
            pillz: CardAttr::default(),
        }
    }
    pub fn print(&self, x: usize, shift_up: bool, playing: bool) {
        let mut shift = if x > 0 {
            format!("\x1b[{}C", x)
        } else {
            String::new()
        };
        let mut up = if shift_up { "\x1b[16A" } else { "" };

        let col = match (playing, self.played, self.won) {
            (true, _, _) => Color::BrightBlue,
            (false, true, false) => Color::BrightRed,
            (false, true, true) => Color::Green,
            _ => Color::BrightWhite,
        };
        println!("{}{}{}", up, shift, "╔════════════════════════╗".color(col));
        for _ in 0..14 {
            println!(
                "{}{b}                        {b}",
                shift,
                b = "║".color(col)
            );
        }
        println!("{}{}", shift, "╚════════════════════════╝".color(col));

        shift = format!("\x1b[{}C", x + 1);
        up = "\x1b[15A";

        let padding = 16 - self.name.len();
        let padding_left = padding * 2 / 3;
        let padding_right = padding - padding_left;
        println!(
            "{}{} {}{}{}{} ",
            up,
            shift,
            " ".repeat(padding_left),
            self.rarity.format_name(&self.name).to_string(),
            " ".repeat(padding_right),
            self.year.to_string().bright_black(),
        );
        println!("{}                        ", shift);
        // println!(
        //     "{} {}{}{}{}  ",
        //     shift,
        //     " ".repeat(20 - self.level_max as usize * 2),
        //     format!("{}", " *".repeat(self.level as usize))
        //         .bright_yellow()
        //         .on_magenta()
        //         .bold(),
        //     format!("{}", " *".repeat((self.level_max - self.level) as usize))
        //         .bright_black()
        //         .on_magenta()
        //         .bold(),
        //     " ".on_magenta(),
        // );
        // if self.power.base == self.power.value {
        //     println!(
        //         "{} {} {:<19}",
        //         shift,
        //         " P ".black().on_blue(),
        //         self.power.base.to_string().blue(),
        //     );
        // } else {
        //     println!(
        //         "{} {} {} {:<16}",
        //         shift,
        //         " P ".black().on_blue(),
        //         self.power.value.to_string().blue().italic(),
        //         self.power.base.to_string().bright_black(),
        //     );
        // }
        if self.power.base == self.power.value {
            print!(
                "{} {} {:<4}",
                shift,
                " P ".black().on_blue(),
                self.power.base.to_string().blue(),
            );
        } else {
            print!(
                "{} {} {}",
                shift,
                " P ".black().on_blue(),
                format!(
                    "{} {}",
                    self.power.base.to_string().bright_black().italic(),
                    self.power.value.to_string().blue().bold(),
                )
            );
        }
        println!(
            "{}{}{}{}{}  ",
            // shift,
            " ".repeat(12 - self.level_max as usize * 2),
            " *".repeat(self.level as usize - 1)
                .bright_yellow()
                .on_magenta()
                .bold(),
            format!(" {}", self.level)
                .bright_yellow()
                .on_magenta()
                .bold(),
            " *".repeat((self.level_max - self.level) as usize)
                .bright_black()
                .on_magenta()
                .bold(),
            " ".on_magenta(),
        );
        if self.damage.base == self.damage.value {
            println!(
                "{} {} {:<19}",
                shift,
                " D ".black().on_red(),
                self.damage.base.to_string().red(),
            );
        } else {
            println!(
                "{} {} {} {:<16}",
                shift,
                " D ".black().on_red().italic(),
                self.damage.base.to_string().bright_black().italic(),
                self.damage.value.to_string().red().bold(),
            );
        }
        // println!("{}                        ", shift);
        println!(
            "{}              {} ",
            shift,
            " Ability ".bright_white().on_blue().italic()
        );
        if let AbilityString::None = self.ability.string {
            println!(
                "{} {:<22}",
                shift,
                " No ability".bright_black().on_bright_white().italic()
            );
            println!("{} {:<22}", shift, "".bright_black().on_bright_white());
            println!("{} {:<22}", shift, "".bright_black().on_bright_white());
        } else {
            for line in split_lines(&self.ability.string.to_string(), 20, 3) {
                println!(
                    "{} {} ",
                    shift,
                    format!(" {} ", line).bright_blue().on_bright_white(),
                );
            }
        }
        println!("{}                        ", shift);
        println!(
            "{}                {} ",
            shift,
            " Bonus ".bright_white().on_red().italic()
        );
        if let AbilityString::None = self.bonus.string {
            println!(
                "{} {:<22}",
                shift,
                " No bonus".bright_black().on_bright_white().italic()
            );
            println!("{} {:<22}", shift, "".bright_black().on_bright_white());
        } else {
            for line in split_lines(&self.bonus.string.to_string(), 20, 2) {
                println!(
                    "{} {} ",
                    shift,
                    format!(" {} ", line).bright_red().on_bright_white(),
                );
            }
        }
        println!("{}                        ", shift);
        println!(
            "{} {} | {:<16}\n",
            shift,
            "Clan".bright_black().italic(),
            self.clan.name(),
        );
    }

    pub fn get_ability(&self) -> Ability {
        ABILITIES[&self.ability_id].clone()
    }
    pub fn get_bonus(&self) -> Ability {
        ABILITIES[&self.bonus_id].clone()
    }
}

#[derive(Debug, Clone)]
pub struct Hand {
    pub cards: [RefCell<Card>; 4],
    pub clan_count: [u8; 4],
    pub oculus_clan: Clan,
}

// impl Index<usize> for Hand {
//     type Output = Card;
//     fn index(&self, index: usize) -> &Self::Output {
//         self.cards[index].borrow().deref()
//     }
// }
// impl IndexMut<usize> for Hand {
//     fn index_mut(&mut self, index: usize) -> &mut Card {
//         self.cards[index].borrow_mut().deref_mut()
//     }
// }

impl Hand {
    pub fn index(&self, index: usize) -> Ref<Card> {
        self.cards[index].borrow()
    }
    // pub fn index_mut(&self, index: usize) -> RefMut<Card> {
    //     self.cards[index].borrow_mut()
    // }
    // pub fn names(&self) -> String {
    //     format!(
    //         "{}, {}, {}, {}",
    //         self.index(0).name,
    //         self.index(1).name,
    //         self.index(2).name,
    //         self.index(3).name
    //     )
    // }
    pub fn print(&self, selected: usize) {
        for (i, card) in self.cards.iter().enumerate() {
            card.borrow().print(i * 28, i > 0, i == selected);
        }
    }
    pub fn card_clan_count(&self, index: usize) -> u8 {
        self.clan_count[index]
    }
    pub fn clan_counts(cards: &[RefCell<Card>; 4]) -> ([u8; 4], Clan) {
        let mut counts = [0u8; 4];
        let mut clans = [Clan::None; 4];
        let mut clan_count = HashMap::<Clan, usize>::with_capacity(4);

        let mut oculus_count = 0;
        let mut oculus_index = 0;
        let mut ids = HashSet::<u32>::with_capacity(4);
        for (i, card) in cards.iter().enumerate() {
            if card.borrow().clan == Clan::Oculus {
                oculus_count += 1;
                oculus_index = i;
            }
            if !ids.contains(&card.borrow().id) {
                clans[i] = card.borrow().clan;
                if clan_count.contains_key(&card.borrow().clan) {
                    *clan_count.get_mut(&card.borrow().clan).unwrap() += 1;
                } else {
                    clan_count.insert(card.borrow().clan, 1);
                }
                ids.insert(card.borrow().id);
            }
        }
        if oculus_count == 1 {
            if clan_count.len() == 2 {
                if oculus_index == 0 {
                    clans[oculus_index] = cards[1].borrow().clan;
                } else {
                    clans[oculus_index] = cards[0].borrow().clan;
                }
            } else if clan_count.len() == 3 {
                let mut solo_clan = Clan::None;
                let mut does_clan_have_2 = false;
                for (&clan, &count) in clan_count.iter() {
                    if count == 2 {
                        does_clan_have_2 = true;
                    } else if count == 1 && clan != Clan::Oculus {
                        solo_clan = clan;
                        if does_clan_have_2 {
                            break;
                        }
                    }
                }
                if does_clan_have_2 {
                    clans[oculus_index] = solo_clan;
                }
            }
        }
        for (i, clan) in clans.iter().enumerate() {
            let mut count = 0;
            for clan1 in clans.iter() {
                if clan == clan1 {
                    count += 1;
                }
            }
            counts[i] = count;
        }

        (counts, clans[oculus_index])
    }
    pub fn get_leader(&self) -> Option<Ref<Card>> {
        if self.index(0).clan == Clan::Leader {
            if self.index(1).clan == Clan::Leader
                || self.index(2).clan == Clan::Leader
                || self.index(3).clan == Clan::Leader
            {
                return None;
            }

            return Some(self.index(0));
        } else if self.index(1).clan == Clan::Leader {
            if self.index(2).clan == Clan::Leader || self.index(3).clan == Clan::Leader {
                return None;
            }

            return Some(self.index(1));
        } else if self.index(2).clan == Clan::Leader {
            if self.index(3).clan == Clan::Leader {
                return None;
            }

            return Some(self.index(2));
        } else if self.index(3).clan == Clan::Leader {
            return Some(self.index(3));
        }

        None
    }
    pub fn random_hand_clan(clan: Clan) -> Self {
        let cards = CARD_CLANS[&clan]
            .choose_multiple(&mut thread_rng(), 4)
            .enumerate()
            .map(|(index, data)| RefCell::new(data.to_card(index)))
            .collect::<Vec<RefCell<Card>>>()
            .try_into()
            .unwrap();
        let (clan_count, oculus_clan) = Hand::clan_counts(&cards);
        Hand {
            cards,
            clan_count,
            oculus_clan,
        }
    }
    pub fn from_ids(i1: u32, i2: u32, i3: u32, i4: u32) -> Self {
        let cards = [
            RefCell::new(CardData::get_id(i1).to_card(0)),
            RefCell::new(CardData::get_id(i2).to_card(1)),
            RefCell::new(CardData::get_id(i3).to_card(2)),
            RefCell::new(CardData::get_id(i4).to_card(3)),
        ];
        let (clan_count, oculus_clan) = Hand::clan_counts(&cards);
        Hand {
            cards,
            clan_count,
            oculus_clan,
        }
    }
    pub fn from_names(c1: &str, c2: &str, c3: &str, c4: &str) -> Self {
        let cards = [
            RefCell::new(CardData::get_name(c1).to_card(0)),
            RefCell::new(CardData::get_name(c2).to_card(1)),
            RefCell::new(CardData::get_name(c3).to_card(2)),
            RefCell::new(CardData::get_name(c4).to_card(3)),
        ];
        let (clan_count, oculus_clan) = Hand::clan_counts(&cards);
        Hand {
            cards,
            clan_count,
            oculus_clan,
        }
    }
}
