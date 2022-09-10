use std::{env, io};

use game::Selection;

use crate::{
    card::Hand,
    game::{Game, GameStatus, PlayerType},
    solver::Solver,
};

mod ability;
mod battle;
mod card;
mod game;
mod modifiers;
mod solver;
mod types;

fn main() {
    let args: Vec<String> = env::args().collect();
    let h1: Hand;
    let h2: Hand;
    let mut flip = 0u8;
    if args.len() >= 9 {
        h1 = Hand::from_names(
            args[1].as_str(),
            args[2].as_str(),
            args[3].as_str(),
            args[4].as_str(),
        );
        h2 = Hand::from_names(
            args[5].as_str(),
            args[6].as_str(),
            args[7].as_str(),
            args[8].as_str(),
        );
        if args.len() == 10 {
            flip = 1;
        }
    } else {
        h1 = Hand::from_names("Anagone", "Doela", "Elios", "Galahad");
        h2 = Hand::from_names("Murray", "Petra", "Buck", "Keile");
    }
    let mut game = Game::new(h1, h2);
    game.flip = flip;

    game.print_status();

    // game.select(0, 0, false);
    // game.select(0, 1, false);

    // game.select(1, 1, false);
    // game.select(1, 2, false);

    // game.select(2, 2, false);
    // game.select(2, 3, true);

    // game.select(1, 1, false);
    // game.select(1, 2, false);

    if flip == 0 {
        println!("{:?}", Solver::solve(&game));
    }

    return;

    println!("{} turn", game.get_turn_name());
    for line in io::stdin().lines() {
        let mut input = line.unwrap();

        if input.as_str() == "cancel" {
            game.clear_selection();
            game.print_status();
            continue;
        } else if input.starts_with("x ") {
            input = input[2..].to_string();
            game.clear_selection();
        }

        let selected = Selection::parse(input);
        if selected.is_none() {
            continue;
        }

        let Selection { index, pillz, fury } = selected.unwrap();

        // println!("{}, {}, {}", index, pillz, fury);
        let battled = game.select(index, pillz, fury);
        if !battled {
            game.print_status();
        }
        if game.status() != GameStatus::Playing {
            break;
        }

        if game.get_turn() == PlayerType::Player {
            println!("{:?}", Solver::solve(&game));
        }

        println!("{} turn", game.get_turn_name());
    }
    game.print_status();
}

#[cfg(test)]
mod test_clan_count {
    use crate::game::Game;

    #[test]
    fn test() {
        let game = Game::random();

        game.print_status();

        println!("{:?}", game.h1.clan_count);
        println!("{:?}", game.h2.clan_count);
    }
}

#[cfg(test)]
mod test1 {
    use regex::{Captures, Regex};

    use crate::{
        ability::{ABILITIES, CLANS_REGEX},
        card::{Card, CARD_IDS},
        types::Clan,
    };

    #[test]
    fn test() {
        println!("{:?}", ABILITIES[&2184]);

        // let s = "Versus [Clan:4][Clan:3]".to_string();
        let s = "[clan:27][clan:42][clan:49][clan:10] +1 Pillz And Life".to_string();
        let re = Regex::new(r"\[Clan:(\d+)\]").unwrap();

        let clans = re
            .captures_iter(s.as_str())
            .map(|m| Clan::from(*&m[1].parse::<u8>().unwrap()))
            .collect::<Vec<Clan>>();
        println!("{:?}", clans);

        let rep = re.replace_all(s.as_str(), |caps: &Captures| {
            Clan::from(*&caps[1].parse::<u8>().unwrap())
                .short_name()
                .to_string()
                + " "
        });
        println!("{}", rep);

        println!("{}", CARD_IDS[&2118].ability);
    }
}

#[cfg(test)]
mod test3 {
    use std::cell::RefCell;

    use lazy_static::__Deref;

    struct A {
        i: RefCell<u8>,
    }

    impl A {
        fn increment(&self) {
            let mut i = self.i.borrow_mut();
            *i += 1;
            if *i < 4 {
                drop(i);
                self.increment();
            }
        }
    }

    fn f(i: &u8) {
        println!("{}", i);
    }

    #[test]
    fn test() {
        let a = A { i: RefCell::new(0) };
        a.increment();
        println!("{}", a.i.borrow());
        f(a.i.borrow().deref());
    }
}

#[cfg(test)]
#[allow(dead_code)]
mod test2 {
    use std::{fs::File, path::Path};

    use serde::Deserialize;
    use serde_repr::Deserialize_repr;
    use simd_json::from_reader;

    #[derive(Debug, Clone, Deserialize)]
    #[serde(rename_all = "UPPERCASE")]
    // #[serde(remote = "EnumString")]
    pub enum EnumString {
        Dog,
        Cat,
    }

    // impl<'de> Deserialize<'de> for EnumString {
    //     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    //     where
    //         D: Deserializer<'de>,
    //     {
    //         let s = String::deserialize(deserializer)?;
    //         EnumString::deserialize(s.into_deserializer())
    //     }
    // }

    #[derive(Debug, Clone, Copy, Deserialize_repr)]
    #[repr(u8)]
    enum TestEnum {
        A = 1,
        B = 2,
    }

    #[derive(Debug, Clone, Copy, Deserialize)]
    struct TestVarStruct {
        j: i8,
    }

    #[derive(Debug, Clone, Copy, Deserialize)]
    // #[serde(tag = "type")]
    #[serde(untagged)]
    enum TestVar {
        C(TestVarStruct),
        D { i: i32 },
        E,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct Test {
        a: Option<TestEnum>,
        b: TestVar,
        c: Option<EnumString>,
    }

    #[test]
    fn test() {
        let data_file =
            File::open(Path::new("./assets/test.json")).expect("file should open read only");
        let json: Vec<Test> = from_reader(data_file).expect("Error while reading JSON file");

        println!("{:?}", json);
    }
}
