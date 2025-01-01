#[cfg(test)]
mod testcases {
    use serde::Deserialize;
    use simd_json::from_reader;

    use std::{fs::File, path::Path};

    use crate::{card::Hand, game::Game};

    #[derive(Clone, Debug, Deserialize)]
    struct Move {
        s1: (usize, u8, bool),
        s2: (usize, u8, bool),
        p1life: u8,
        p2life: u8,
        p1pillz: u8,
        p2pillz: u8,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct Testcase {
        cards: [String; 8],
        flip: bool,
        life: u8,
        pillz: u8,
        moves: Vec<Move>,
    }

    #[test]
    fn testcases() {
        let data_file = File::open(Path::new("./assets/testcases10000.json")).unwrap();
        let json: Vec<Testcase> = from_reader(data_file).unwrap();

        for (i, t) in json.into_iter().enumerate() {
            let h1 = Hand::from_names(
                t.cards[0].as_str(),
                t.cards[1].as_str(),
                t.cards[2].as_str(),
                t.cards[3].as_str(),
            );
            let h2 = Hand::from_names(
                t.cards[4].as_str(),
                t.cards[5].as_str(),
                t.cards[6].as_str(),
                t.cards[7].as_str(),
            );

            let mut game = Game::new(h1, h2);
            game.flip = t.flip as u8;
            game.p1.life = t.life;
            game.p2.life = t.life;
            game.p1.pillz = t.pillz;
            game.p2.pillz = t.pillz;

            game.print_status();

            for Move {
                s1,
                s2,
                p1life,
                p2life,
                p1pillz,
                p2pillz,
            } in t.moves
            {
                game.select(s1.0, s1.1, s1.2);
                game.select(s2.0, s2.1, s2.2);

                assert_eq!(
                    game.p1.life, p1life,
                    "Testcase {}: p1life should be: {}",
                    i, p1life
                );
                assert_eq!(
                    game.p2.life, p2life,
                    "Testcase {}: p2life should be: {}",
                    i, p2life
                );
                assert_eq!(
                    game.p1.pillz, p1pillz,
                    "Testcase {}: p1pillz should be: {}",
                    i, p1pillz
                );
                assert_eq!(
                    game.p2.pillz, p2pillz,
                    "Testcase {}: p2pillz should be: {}",
                    i, p2pillz
                );

                game.print_status();
            }
        }
    }
}
