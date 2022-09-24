use std::{
    env,
    io::{self, Result},
};

use game::Selection;
use rayon::ThreadPoolBuilder;

use crate::{
    card::Hand,
    game::{Game, GameStatus, PlayerType},
    solver::{SelectionResult, Solver},
};

mod ability;
mod battle;
mod card;
mod game;
mod modifiers;
mod server;
mod solver;
mod types;
mod utils;

#[allow(unreachable_code)]
#[actix_web::main]
async fn main() -> Result<()> {
    ThreadPoolBuilder::new()
        .num_threads(4)
        .build_global()
        .unwrap();
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
        // h1 = Hand::from_names("Anagone", "Doela", "Elios", "Galahad");
        // h2 = Hand::from_names("Murray", "Petra", "Buck", "Keile");

        server::serve().await?;

        return Ok(());
    }
    let mut game = Game::new(h1, h2);
    game.flip = flip;

    game.print_status();

    if flip == 0 {
        // let best = Solver::solve(&game);

        // match (best, game.get_turn()) {
        //     (SelectionResult::Player(_), PlayerType::Opponent)
        //     | (SelectionResult::Opponent(_), PlayerType::Player) => {
        //         Solver::middle(&game);
        //     }
        //     (_, _) => println!("{:?}", best),
        // }
        Solver::middle(&game);
    }

    // return;

    println!("{} turn", game.get_turn_name());
    for line in io::stdin().lines() {
        let mut input = line.unwrap();

        let cancelled: bool;
        if input.as_str() == "cancel" {
            game.clear_selection();
            game.print_status();
            // cancelled = true;
            continue;
        } else if input.starts_with("x ") {
            input = input[2..].to_string();
            game.clear_selection();
            cancelled = true;
        } else {
            cancelled = false;
        }

        let selected = Selection::parse(input);
        if selected.is_none() {
            continue;
        }

        let Selection { index, pillz, fury } = selected.unwrap();

        // println!("{}, {}, {}", index, pillz, fury);
        if !game.can_select(index, pillz, fury) {
            continue;
        }

        let battled = game.select(index, pillz, fury);
        if !battled {
            game.print_status();
        }
        if game.status() != GameStatus::Playing {
            break;
        }

        let turn = game.get_turn();

        if game.round == 0 {
            if !cancelled && turn == PlayerType::Player {
                Solver::middle(&game);
            }
        } else {
            let best = Solver::solve(&game);

            match (best, turn) {
                (SelectionResult::Player(_), PlayerType::Opponent)
                | (SelectionResult::Opponent(_), PlayerType::Player) => {
                    Solver::middle(&game);
                }
                (_, _) => println!("{:?}", best),
            }
        }

        println!("{} turn", game.get_turn_name());
    }
    game.print_status();

    Ok(())
}

// #[cfg(test)]
// mod test_clan_count {
//     use crate::{card::Hand, types::Clan};

//     #[test]
//     fn test() {
//         // let game = Game::random();

//         // game.print_status();

//         // println!("{:?}", game.h1.clan_count);
//         // println!("{:?}", game.h2.clan_count);
//         let h1 = Hand::random_hand_clan(Clan::AllStars);
//         let h2 = h1.clone();

//         // let mut card1 = h1.cards[0].borrow_mut();
//         // let card2 = h2.cards[0].borrow_mut();
//         // card1.played = true;

//         // println!("{:#?}\n{:#?}", card1, card2);
//     }
// }

#[cfg(test)]
mod test1 {
    use regex::{Captures, Regex};

    use crate::{ability::ABILITIES, card::CARD_IDS, types::Clan};

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

// #[cfg(test)]
// mod test4 {
//     use std::{
//         cell::RefCell,
//         sync::{Arc, Mutex},
//         thread,
//         time::Duration,
//     };

//     #[derive(Debug, Clone)]
//     struct Struct {
//         a: RefCell<u32>,
//     }

//     impl Struct {
//         fn new(a: u32) -> Self {
//             Self { a: RefCell::new(a) }
//         }
//     }

//     #[test]
//     fn test() {
//         let a = Arc::new(Mutex::new(Struct::new(0)));
//         println!("{:?}", a);

//         let mut handlers = Vec::new();
//         for i in 0..4 {
//             let handler = thread::scope(|s| {
//                 s.spawn(|| {
//                     let num = i.clone();
//                     thread::sleep(Duration::from_millis(500));
//                     let b = a.clone().lock().unwrap().clone();
//                     *b.a.borrow_mut() += 1;
//                     println!("h1 {:?}", b);
//                     return num;
//                 });
//             });
//             handlers.push(handler);
//         }
//         let mut best = 0;
//         for handler in handlers {
//             let result = handler.join().unwrap();
//             if result > best {
//                 best = result;
//             }
//         }
//         println!("{}", best);
//     }
// }

#[cfg(test)]
mod test5 {
    use std::{
        cell::RefCell,
        sync::{Arc, Mutex},
        thread,
        time::Duration,
    };

    use rayon::prelude::*;

    #[derive(Debug, Clone)]
    struct Test {
        a: RefCell<i32>,
    }

    impl Test {
        fn new(a: i32) -> Self {
            Self { a: RefCell::new(a) }
        }
    }

    #[test]
    fn test() {
        let a = 5;
        // let mut counter = 0;

        let s = Arc::new(Mutex::new(Test::new(0)));
        (0..4).into_par_iter().for_each(|i| {
            thread::sleep(Duration::from_millis(i * 300));
            let x = s.lock().unwrap().clone();
            *x.a.borrow_mut() += a;
            println!("{:?}", x);

            // for _ in 0..1000000 {
            //     counter += 1;
            // }
        });

        // println!("{}", counter);
    }
}

#[cfg(test)]
mod test6 {
    use tinyvec::ArrayVec;

    #[derive(Debug, Default, Clone, Copy)]
    struct Test {
        a: u8,
    }

    #[test]
    fn test() {
        let mut vec = ArrayVec::<[Test; 4]>::new();
        vec.push(Test { a: 1 });

        let mut vec1 = vec;
        vec1[0].a += 1;

        println!("{:?}", vec);
        println!("{:?}", vec1);
    }
}

#[cfg(test)]
mod test7 {
    use std::borrow::Cow;

    #[derive(Debug, Default, Clone)]
    struct Test<'a> {
        a: Cow<'a, Option<bool>>,
    }

    #[test]
    fn test() {
        let mut a = Test {
            a: Cow::Owned(None),
        };
        let mut b = a.clone();
        let c = a.clone();
        println!("{:?} {:?} {:?}", a.a, b.a, c.a);

        *a.a.to_mut() = Some(false);
        *b.a.to_mut() = Some(true);
        println!("{:?} {:?} {:?}", a.a, b.a, c.a);
    }
}
