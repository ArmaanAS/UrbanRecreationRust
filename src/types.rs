use core::panic;
use std::fmt::{Display, Formatter, Result};

use colored::{Color, ColoredString, Colorize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(Clone, Copy, Serialize_repr, Deserialize_repr, PartialEq, Eq, Debug, Hash)]
#[repr(u8)]
pub enum Clan {
    None = 0,
    AllStars = 38,
    Bangers = 31,
    Berzerk = 46,
    Cosmohnuts = 58,
    Dominion = 53,
    FangPiClang = 25,
    Freaks = 40,
    Frozn = 47,
    GHEIST = 32,
    GhosTown = 52,
    Hive = 51,
    Huracan = 48,
    Jungo = 43,
    Junkz = 26,
    Komboka = 54,
    LaJunta = 27,
    Leader = 36,
    Montana = 3,
    Nightmare = 37,
    Oblivion = 57,
    Oculus = 56,
    Paradox = 55,
    Piranas = 42,
    Pussycats = 4,
    Raptors = 50,
    Rescue = 41,
    Riots = 49,
    Roots = 29,
    Sakrohm = 30,
    Sentinel = 33,
    Skeelz = 44,
    UluWatu = 10,
    Uppers = 28,
    Vortex = 45,
    Zenith = 59,
}

impl From<u8> for Clan {
    #[inline]
    fn from(clan: u8) -> Self {
        match clan {
            38 => Clan::AllStars,
            31 => Clan::Bangers,
            46 => Clan::Berzerk,
            58 => Clan::Cosmohnuts,
            53 => Clan::Dominion,
            25 => Clan::FangPiClang,
            40 => Clan::Freaks,
            47 => Clan::Frozn,
            32 => Clan::GHEIST,
            52 => Clan::GhosTown,
            51 => Clan::Hive,
            48 => Clan::Huracan,
            43 => Clan::Jungo,
            26 => Clan::Junkz,
            54 => Clan::Komboka,
            27 => Clan::LaJunta,
            36 => Clan::Leader,
            3 => Clan::Montana,
            37 => Clan::Nightmare,
            57 => Clan::Oblivion,
            56 => Clan::Oculus,
            55 => Clan::Paradox,
            42 => Clan::Piranas,
            4 => Clan::Pussycats,
            50 => Clan::Raptors,
            41 => Clan::Rescue,
            49 => Clan::Riots,
            29 => Clan::Roots,
            30 => Clan::Sakrohm,
            33 => Clan::Sentinel,
            44 => Clan::Skeelz,
            10 => Clan::UluWatu,
            28 => Clan::Uppers,
            45 => Clan::Vortex,
            59 => Clan::Zenith,
            _ => panic!("Invalid clan id: {}", clan),
        }
    }
}

impl Clan {
    #[inline]
    fn color(&self) -> Color {
        match self {
            Clan::AllStars | Clan::GhosTown => Color::BrightBlue,
            Clan::Freaks | Clan::Sakrohm | Clan::UluWatu | Clan::Uppers => Color::BrightGreen,
            Clan::Roots | Clan::Cosmohnuts => Color::Green,
            Clan::Montana | Clan::Pussycats => Color::BrightMagenta,
            Clan::Paradox | Clan::Skeelz | Clan::Dominion => Color::Magenta,
            Clan::Frozn => Color::BrightCyan,
            Clan::Komboka => Color::Cyan,
            Clan::Berzerk | Clan::FangPiClang | Clan::Huracan | Clan::Leader => Color::BrightRed,
            Clan::GHEIST | Clan::Oculus => Color::Red,
            Clan::Hive | Clan::Junkz | Clan::Piranas | Clan::Rescue => Color::BrightYellow,
            Clan::Bangers
            | Clan::Jungo
            | Clan::LaJunta
            | Clan::Raptors
            | Clan::Riots
            | Clan::Sentinel => Color::Yellow,
            Clan::Vortex | Clan::Nightmare => Color::BrightBlack,
            Clan::Oblivion | Clan::Zenith => Color::BrightWhite,
            Clan::None => unreachable!(),
        }
    }
    #[inline]
    pub fn name(&self) -> ColoredString {
        format!("{:?}", self).color(self.color())
    }
    #[inline]
    pub fn short_name(&self) -> String {
        format!(
            "[{}]",
            match self {
                Clan::AllStars => "AllS",
                Clan::Bangers => "Bngr",
                Clan::Berzerk => "Bzrk",
                Clan::Cosmohnuts => "Csmo",
                Clan::Dominion => "Domn",
                Clan::FangPiClang => "Fng",
                Clan::Freaks => "Frks",
                Clan::Frozn => "Frzn",
                Clan::GHEIST => "Ghst",
                Clan::GhosTown => "GhTn",
                Clan::Hive => "Hive",
                Clan::Huracan => "Hrcn",
                Clan::Jungo => "Jngo",
                Clan::Junkz => "Jnkz",
                Clan::Komboka => "Kmbk",
                Clan::LaJunta => "LaJn",
                Clan::Leader => "Ldr",
                Clan::Montana => "Mtna",
                Clan::Nightmare => "Ntmr",
                Clan::Oblivion => "Oblv",
                Clan::Oculus => "Ocul",
                Clan::Paradox => "Prdx",
                Clan::Piranas => "Prna",
                Clan::Pussycats => "Pssy",
                Clan::Raptors => "Rptr",
                Clan::Rescue => "Rscu",
                Clan::Riots => "Riot",
                Clan::Roots => "Root",
                Clan::Sakrohm => "Sakm",
                Clan::Sentinel => "Sntl",
                Clan::Skeelz => "Sklz",
                Clan::UluWatu => "UluW",
                Clan::Uppers => "Uprs",
                Clan::Vortex => "Vrtx",
                Clan::Zenith => "Znth",
                Clan::None => unreachable!(),
            }
        )
        // .color(self.color())
    }
}

impl Display for Clan {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Rarity {
    COMMON,
    UNCOMMON,
    RARE,
    COLLECTOR,
    MYTHIC,
    LEGENDARY,
}

impl Rarity {
    #[inline]
    pub fn from(s: &String) -> Self {
        match s.as_str() {
            "c" => Rarity::COMMON,
            "u" => Rarity::UNCOMMON,
            "r" => Rarity::RARE,
            "cr" => Rarity::COLLECTOR,
            "m" => Rarity::MYTHIC,
            "l" => Rarity::LEGENDARY,
            _ => Rarity::COMMON,
        }
    }
    #[inline]
    pub fn format_name(&self, name: &String) -> ColoredString {
        match self {
            Rarity::COMMON => format!(" {} ", name).bright_white().on_red(),
            Rarity::UNCOMMON => format!(" {} ", name).bright_black().on_bright_white(),
            Rarity::RARE => format!(" {} ", name).black().on_bright_yellow(),
            Rarity::COLLECTOR => format!(" {} ", name).bright_yellow().on_black(),
            Rarity::MYTHIC => format!(" {} ", name).bright_white().on_bright_blue(),
            Rarity::LEGENDARY => format!(" {} ", name).bright_white().on_bright_magenta(),
        }
    }
}
