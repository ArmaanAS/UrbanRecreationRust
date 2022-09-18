use std::{
    io::{stdout, Write},
    slice::Iter,
    time::Instant,
};

use colored::{Color, Colorize};
use lazy_static::lazy_static;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

use crate::{
    ability, battle,
    game::{self, Game, GameStatus, PlayerType, Selection, BATTLE_COUNT},
    modifiers,
};

static mut SOLVE_COUNT: u64 = 0;

pub struct Solver {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameResult {
    Win,
    Draw,
    Lose,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelectionResult {
    Player(Selection),
    Draw(Selection),
    Opponent(Selection),
}

pub fn toggle_print() {
    unsafe {
        ability::PRINT = !ability::PRINT;
        game::PRINT = !game::PRINT;
        modifiers::PRINT = !modifiers::PRINT;
        battle::PRINT = !battle::PRINT;
    }
}

impl Solver {
    pub fn middle(game: &Game) {
        let battle_count = unsafe { BATTLE_COUNT };
        toggle_print();
        let now = Instant::now();
        if game.s1.is_some() || game.s2.is_some() {
            if game.round == 0 {
                Solver::middle_second_par(game);
            } else {
                Solver::middle_second(game);
            }
        } else if game.round == 0 {
            Solver::middle_first_par(game);
        } else {
            Solver::middle_first(game);
        }
        toggle_print();
        unsafe {
            let battles = BATTLE_COUNT - battle_count;
            let elapsed = now.elapsed();
            println!(
                "{} {} /{:.1?}secs  ({:.0?}k/s)",
                " Battle Count ".white().on_bright_purple(),
                battles,
                elapsed.as_secs_f32(),
                battles as f32 / elapsed.as_secs_f32() / 1000f32
            );
        }
    }

    fn middle_second(game: &Game) {
        let i = if game.s1.is_none() {
            game.s2.unwrap().index
        } else {
            game.s1.unwrap().index
        };

        let pillz1 = game.get_turn_opponent().pillz;
        let pillz2 = game.get_turn_player().pillz;

        let turn = game.get_turn();
        let hand = game.get_turn_hand();

        let mut game = game.clone();
        game.clear_selection();

        let mut best_pillz = 0;
        let mut best_rate = 0f32;
        let mut best_rate_rounded = 0u32;
        let mut best_selection = Selection::default();

        for index in 0..4 {
            if hand.index(index).played {
                continue;
            }

            for &(pillz, fury) in shift_false_range(pillz2, game.round) {
                let mut p_wins = 0u8;
                let mut draws = 0u8;
                let mut o_wins = 0u8;

                for &(p, f) in split_range(pillz1) {
                    let mut g = game.clone();
                    g.select(i, p, f);
                    g.select(index, pillz, fury);

                    match g.status() {
                        GameStatus::Player => p_wins += 1,
                        GameStatus::Draw => draws += 1,
                        GameStatus::Opponent => o_wins += 1,
                        GameStatus::Playing => {
                            let best = Solver::solve_first(&g);
                            match best {
                                SelectionResult::Player(_) => p_wins += 1,
                                SelectionResult::Draw(_) => draws += 1,
                                SelectionResult::Opponent(_) => o_wins += 1,
                            }
                        }
                    }
                }
                let (wins, losses) = if turn == PlayerType::Player {
                    (p_wins, o_wins)
                } else {
                    (o_wins, p_wins)
                };

                let rate = (wins + draws) as f32 / (wins + draws + losses) as f32;
                let rate_rounded = (rate * 100f32) as u32 / 10;
                if rate_rounded > best_rate_rounded
                    || (rate_rounded == best_rate_rounded && pillz < best_pillz)
                {
                    best_pillz = pillz;
                    best_rate = rate;
                    best_rate_rounded = rate_rounded;
                    best_selection = Selection::new(index, pillz, fury);
                }

                if losses == 0 {
                    if draws == 0 {
                        print!("{} ", pillz.to_string().black().on_green());
                    } else {
                        print!("{} ", "d".bright_yellow());
                    }
                } else if wins + draws > losses {
                    if wins == 0 {
                        print!("{} ", "d".bright_yellow());
                    } else {
                        print!(
                            "{} ",
                            format!("{:X}", pillz).color(if fury {
                                Color::Red
                            } else {
                                Color::Green
                            })
                        )
                    }
                } else if rate <= 0.25 {
                    print!("{} ", "x".bright_black())
                } else if wins + draws <= losses {
                    print!("{} ", format!("{:X}", pillz).bright_black())
                } else {
                    println!("({}, {}, {})", wins, losses, draws);
                }
                stdout().flush().unwrap();
            }
            // println!();
            println!("({:.1?}%) {:?}", best_rate * 100f32, best_selection);
        }

        println!("({:.1?}%) {:?}", best_rate * 100f32, best_selection);
    }

    fn middle_second_par(game: &Game) {
        let i = if game.s1.is_none() {
            game.s2.unwrap().index
        } else {
            game.s1.unwrap().index
        };

        let turn = game.get_turn();
        let pillz1 = game.get_turn_opponent().pillz;
        let pillz2 = game.get_turn_player().pillz;

        let mut game = game.clone();
        game.clear_selection();

        let (best_rate, best_selection) = (0..4)
            // .filter(|&index| !game.get_turn_hand().index(index).played)
            // .collect::<Vec<usize>>()
            .into_par_iter()
            .map(|index| {
                let game = game.clone();

                let mut best_pillz = 0;
                let mut best_rate = 0f32;
                let mut best_rate_rounded = 0u32;
                let mut best_selection = Selection::default();

                for &(pillz, fury) in shift_false_range(pillz2, game.round) {
                    let mut p_wins = 0u8;
                    let mut draws = 0u8;
                    let mut o_wins = 0u8;

                    for &(p, f) in split_range(pillz1) {
                        if p == 0 {
                            continue;
                        }
                        let mut g = game.clone();
                        g.select(i, p, f);
                        g.select(index, pillz, fury);

                        match g.status() {
                            GameStatus::Player => p_wins += 1,
                            GameStatus::Draw => draws += 1,
                            GameStatus::Opponent => o_wins += 1,
                            GameStatus::Playing => {
                                let best = Solver::solve_first(&g);
                                match best {
                                    SelectionResult::Player(_) => p_wins += 1,
                                    SelectionResult::Draw(_) => draws += 1,
                                    SelectionResult::Opponent(_) => o_wins += 1,
                                }
                            }
                        }
                    }
                    let (wins, losses) = if turn == PlayerType::Player {
                        (p_wins, o_wins)
                    } else {
                        (o_wins, p_wins)
                    };

                    let rate = (wins + draws) as f32 / (wins + draws + losses) as f32;
                    let rate_rounded = (rate * 100f32) as u32 / 10;
                    if rate_rounded > best_rate_rounded
                        || (rate_rounded == best_rate_rounded && pillz < best_pillz)
                    {
                        best_pillz = pillz;
                        best_rate = rate;
                        best_rate_rounded = rate_rounded;
                        best_selection = Selection::new(index, pillz, fury);
                    }

                    if losses == 0 {
                        if draws == 0 {
                            print!("{} ", pillz.to_string().black().on_green());
                        } else {
                            print!("{} ", "d".bright_yellow());
                        }
                    } else if wins + draws > losses {
                        if wins == 0 {
                            print!("{} ", "d".bright_yellow());
                        } else {
                            print!(
                                "{} ",
                                format!("{:X}", pillz).color(if fury {
                                    Color::Red
                                } else {
                                    Color::Green
                                })
                            )
                        }
                    } else if rate <= 0.25 {
                        print!("{} ", "x".bright_black())
                    } else if wins + draws <= losses {
                        print!("{} ", format!("{:X}", pillz).bright_black())
                    } else {
                        println!("({}, {}, {})", wins, losses, draws);
                    }
                    stdout().flush().unwrap();
                }
                // println!();
                println!("\n({:.1?}%) {:?}", best_rate * 100f32, best_selection);

                (best_rate, best_selection)
            })
            .max_by_key(|&(rate, _)| (rate * 1000f32) as u32)
            .unwrap();

        println!(
            "{}{}",
            format!(" {:.1?}% ", best_rate * 100f32).black().on_green(),
            format!(" {:?} ", best_selection).green()
        );
    }

    fn middle_first(game: &Game) {
        let pillz1 = game.get_turn_player().pillz;
        let pillz2 = game.get_turn_opponent().pillz;

        let turn = game.get_turn();
        let hand1 = game.get_turn_hand();
        let hand2 = game.get_turn_opponent_hand();

        let mut best_pillz = 0;
        let mut best_rate = 0f32;
        let mut best_rate_rounded = 0u32;
        let mut best_selection = Selection::default();

        for index in 0..4 {
            if hand1.index(index).played {
                continue;
            }

            for &(pillz, fury) in shift_false_range(pillz1, game.round) {
                let mut p_wins = 0;
                let mut draws = 0;
                let mut o_wins = 0;

                for i in 0..4 {
                    if hand2.index(i).played {
                        continue;
                    }

                    for &(p, f) in split_range(pillz2) {
                        let mut g = game.clone();
                        g.select(index, pillz, fury);
                        g.select(i, p, f);

                        match g.status() {
                            GameStatus::Player => p_wins += 1,
                            GameStatus::Draw => draws += 1,
                            GameStatus::Opponent => o_wins += 1,
                            GameStatus::Playing => {
                                let best = Solver::solve_first(&g);
                                match best {
                                    SelectionResult::Player(_) => p_wins += 1,
                                    SelectionResult::Draw(_) => draws += 1,
                                    SelectionResult::Opponent(_) => o_wins += 1,
                                }
                            }
                        }
                    }
                }
                let (wins, losses) = if turn == PlayerType::Player {
                    (p_wins, o_wins)
                } else {
                    (o_wins, p_wins)
                };

                let rate = (wins + draws) as f32 / (wins + draws + losses) as f32;
                let rate_rounded = (rate * 100f32) as u32 / 10;
                if rate_rounded > best_rate_rounded
                    || (rate_rounded == best_rate_rounded && pillz < best_pillz)
                {
                    best_pillz = pillz;
                    best_rate = rate;
                    best_rate_rounded = rate_rounded;
                    best_selection = Selection::new(index, pillz, fury);
                }

                if losses == 0 {
                    if draws == 0 {
                        print!("{} ", pillz.to_string().black().on_green());
                    } else {
                        print!("{} ", "d".bright_yellow());
                    }
                } else if wins + draws > losses {
                    if wins == 0 {
                        print!("{} ", "d".bright_yellow());
                    } else {
                        print!(
                            "{} ",
                            format!("{:X}", pillz).color(if fury {
                                Color::Red
                            } else {
                                Color::Green
                            })
                        )
                    }
                } else if rate <= 0.25 {
                    print!("{} ", "x".bright_black())
                } else if wins + draws <= losses {
                    print!("{} ", format!("{:X}", pillz).bright_black())
                } else {
                    println!("({}, {}, {})", wins, losses, draws);
                }
                stdout().flush().unwrap();
            }
            // println!();
            println!("({:.1?}%) {:?}", best_rate * 100f32, best_selection);
        }

        println!("({:.1?}%) {:?}", best_rate * 100f32, best_selection);
    }

    fn middle_first_par(game: &Game) {
        let (best_rate, best_selection) = (0..4)
            // .filter(|&index| !game.get_turn_hand().index(index).played)
            // .collect::<Vec<usize>>()
            .into_par_iter()
            .map(|index| {
                let game = game.clone();
                // let hand1 = game.get_turn_hand();
                let hand2 = game.get_turn_opponent_hand();
                let pillz1 = game.get_turn_player().pillz;
                let pillz2 = game.get_turn_opponent().pillz;
                let turn = game.get_turn();

                let mut best_pillz = 0;
                let mut best_rate = 0f32;
                let mut best_rate_rounded = 0u32;
                let mut best_selection = Selection::default();

                for &(pillz, fury) in shift_false_range(pillz1, game.round) {
                    let mut p_wins = 0;
                    let mut draws = 0;
                    let mut o_wins = 0;

                    for i in 0..4 {
                        if hand2.index(i).played {
                            continue;
                        }

                        for &(p, f) in split_range(pillz2) {
                            if p == 0 {
                                continue;
                            }
                            let mut g = game.clone();
                            g.select(index, pillz, fury);
                            g.select(i, p, f);

                            match g.status() {
                                GameStatus::Player => p_wins += 1,
                                GameStatus::Draw => draws += 1,
                                GameStatus::Opponent => o_wins += 1,
                                GameStatus::Playing => {
                                    let best = Solver::solve_first(&g);
                                    match best {
                                        SelectionResult::Player(_) => p_wins += 1,
                                        SelectionResult::Draw(_) => draws += 1,
                                        SelectionResult::Opponent(_) => o_wins += 1,
                                    }
                                }
                            }
                        }
                    }
                    let (wins, losses) = if turn == PlayerType::Player {
                        (p_wins, o_wins)
                    } else {
                        (o_wins, p_wins)
                    };

                    let rate = (wins + draws) as f32 / (wins + draws + losses) as f32;
                    let rate_rounded = (rate * 100f32) as u32 / 10;
                    if rate_rounded > best_rate_rounded
                        || (rate_rounded == best_rate_rounded && pillz < best_pillz)
                    {
                        best_pillz = pillz;
                        best_rate = rate;
                        best_rate_rounded = rate_rounded;
                        best_selection = Selection::new(index, pillz, fury);
                    }

                    if losses == 0 {
                        if draws == 0 {
                            print!("{} ", pillz.to_string().black().on_green());
                        } else {
                            print!("{} ", "d".bright_yellow());
                        }
                    } else if wins + draws > losses {
                        if wins == 0 {
                            print!("{} ", "d".bright_yellow());
                        } else {
                            print!(
                                "{} ",
                                format!("{:X}", pillz).color(if fury {
                                    Color::Red
                                } else {
                                    Color::Green
                                })
                            )
                        }
                    } else if rate <= 0.25 {
                        print!("{} ", "x".bright_black())
                    } else if wins + draws <= losses {
                        print!("{} ", format!("{:X}", pillz).bright_black())
                    } else {
                        println!("({}, {}, {})", wins, losses, draws);
                    }
                    // stdout().flush().unwrap();
                }
                // println!();
                println!("\n({:.1?}%) {:?}", best_rate * 100f32, best_selection);

                (best_rate, best_selection)
            })
            .max_by_key(|&(rate, _)| (rate * 1000f32) as u32)
            .unwrap();

        println!(
            "{}{}",
            format!(" {:.1?}% ", best_rate * 100f32).black().on_green(),
            format!(" {:?} ", best_selection).green()
        );
    }

    // pub fn middle_first(game: &Game) {
    //     let turn = game.get_turn();
    //     let pillz = game.get_turn_player().pillz;
    //     let hand = game.get_turn_hand();

    //     for index in 0..4 {
    //         if hand.index(index).played {
    //             continue;
    //         }

    //         for &(pillz, fury) in split_range(pillz) {
    //             let mut g = game.clone();
    //             let battled = g.select(index, pillz, fury);

    //             let status = if battled {
    //                 g.status()
    //             } else {
    //                 GameStatus::Playing
    //             };
    //             let win: PlayerResult;
    //             match status {
    //                 GameStatus::Player => win = PlayerResult::Player,
    //                 GameStatus::Draw => win = PlayerResult::Draw,
    //                 GameStatus::Opponent => win = PlayerResult::Opponent,
    //                 GameStatus::Playing => {
    //                     let best = Solver::solve_first(&g);
    //                     match best {
    //                         SelectionResult::Player(_) => win = PlayerResult::Player,
    //                         SelectionResult::Draw(_) => win = PlayerResult::Draw,
    //                         SelectionResult::Opponent(_) => win = PlayerResult::Opponent,
    //                     }
    //                 }
    //             }
    //             match (win, turn) {
    //                 (PlayerResult::Player, PlayerType::Player)
    //                 | (PlayerResult::Opponent, PlayerType::Opponent) => print!(
    //                     "{} ",
    //                     format!("{:X}", pillz).color(if fury { Color::Red } else { Color::Green })
    //                 ),
    //                 (PlayerResult::Draw, _) => print!("{} ", "d".bright_yellow()),
    //                 (_, _) => print!("{} ", "x".bright_black()),
    //             }
    //         }
    //         println!();
    //     }
    // }

    pub fn solve(game: &Game) -> SelectionResult {
        let solve_count: u64;
        let battle_count: u32;
        toggle_print();
        unsafe {
            solve_count = SOLVE_COUNT;
            battle_count = BATTLE_COUNT;
        }
        let now = Instant::now();
        let best = if game.s1.is_none() != game.s2.is_none() {
            Solver::solve_second(game)
        } else {
            Solver::solve_first(game)
        };
        toggle_print();
        unsafe {
            let solves = SOLVE_COUNT - solve_count;
            let battles = BATTLE_COUNT - battle_count;
            let elapsed = now.elapsed();
            println!(
                "{} {} /{:.1?}secs ({:.0?}k/s) - Final count",
                " Solve Count ".white().on_magenta(),
                solves,
                elapsed.as_secs_f32(),
                solves as f32 / elapsed.as_secs_f32() / 1000f32
            );
            println!(
                "{} {} /{:.1?}secs ({:.0?}k/s) - Final count",
                " Battle Count ".white().on_bright_purple(),
                battles,
                elapsed.as_secs_f32(),
                battles as f32 / elapsed.as_secs_f32() / 1000f32
            );
        }
        // handler.join().unwrap();
        best
    }

    pub fn solve_second(game: &Game) -> SelectionResult {
        unsafe {
            SOLVE_COUNT += 1;
        }

        let turn = game.get_turn();
        let i = if game.s1.is_none() {
            game.s2.unwrap().index
        } else {
            game.s1.unwrap().index
        };

        let pillz1 = game.get_turn_player().pillz;
        let pillz2 = game.get_turn_opponent().pillz;

        let mut game = game.clone();
        game.clear_selection();

        let mut worst_result: Option<SelectionResult> = None;

        for index in 0..4usize {
            if game.get_turn_opponent_hand().index(index).played {
                continue;
            }

            for &(pillz, fury) in split_shift_range(pillz1) {
                let mut worst = GameResult::Win;
                for &(p, f) in split_shift_range(pillz2) {
                    let mut g = game.clone();
                    g.select(i, p, f);
                    g.select(index, pillz, fury);

                    match (g.status(), turn) {
                        (GameStatus::Player, PlayerType::Opponent)
                        | (GameStatus::Opponent, PlayerType::Player) => {
                            worst = GameResult::Lose;
                            break;
                        }
                        (GameStatus::Draw, _) => {
                            worst = GameResult::Draw;
                            continue;
                        }
                        (GameStatus::Player, _) | (GameStatus::Opponent, _) => continue,
                        (GameStatus::Playing, _) => (),
                    }

                    let best = Solver::solve_first(&g);

                    match (best, turn) {
                        (SelectionResult::Draw(_), _) => {
                            worst = GameResult::Draw;
                        }
                        (SelectionResult::Opponent(_), PlayerType::Player)
                        | (SelectionResult::Player(_), PlayerType::Opponent) => {
                            worst = GameResult::Lose;
                            break;
                        }
                        (_, _) => (),
                    }
                }

                let selection = Selection { index, pillz, fury };
                if worst == GameResult::Win {
                    return if turn == PlayerType::Player {
                        SelectionResult::Player(selection)
                    } else {
                        SelectionResult::Opponent(selection)
                    };
                } else if worst == GameResult::Draw {
                    worst_result = Some(SelectionResult::Draw(selection));
                } else if worst_result.is_none() {
                    if turn == PlayerType::Player {
                        worst_result = Some(SelectionResult::Opponent(selection))
                    } else {
                        worst_result = Some(SelectionResult::Player(selection))
                    }
                }
            }
        }

        worst_result.unwrap()
    }

    pub fn solve_first(game: &Game) -> SelectionResult {
        unsafe {
            SOLVE_COUNT += 1;
        }
        let turn = game.get_turn();
        let mut result: Option<SelectionResult> = None;

        for index in 0..4usize {
            if game.get_turn_hand().index(index).played {
                continue;
            }

            for &(pillz, fury) in split_shift_range(game.get_turn_player().pillz) {
                let mut g = game.clone();

                let battled = g.select(index, pillz, fury);

                let status = g.status();
                if battled && status != GameStatus::Playing {
                    match (status, turn) {
                        (GameStatus::Draw, _) => match result {
                            None
                            | Some(SelectionResult::Opponent(_))
                            | Some(SelectionResult::Player(_)) => {
                                result =
                                    Some(SelectionResult::Draw(Selection { index, pillz, fury }));
                            }
                            _ => (),
                        },
                        (GameStatus::Opponent, PlayerType::Opponent) => {
                            return SelectionResult::Opponent(Selection { index, pillz, fury });
                        }
                        (GameStatus::Player, PlayerType::Player) => {
                            return SelectionResult::Player(Selection { index, pillz, fury });
                        }
                        (GameStatus::Opponent, PlayerType::Player) => {
                            if result.is_none() {
                                result = Some(SelectionResult::Opponent(Selection {
                                    index,
                                    pillz,
                                    fury,
                                }));
                            }
                        }
                        (GameStatus::Player, PlayerType::Opponent) => {
                            if result.is_none() {
                                result =
                                    Some(SelectionResult::Player(Selection { index, pillz, fury }));
                            }
                        }
                        _ => (),
                    }
                } else {
                    let best = Solver::solve_first(&g);
                    match (best, turn) {
                        (SelectionResult::Draw(_), _) => {
                            if result.is_none() {
                                result =
                                    Some(SelectionResult::Draw(Selection { index, pillz, fury }));
                            }
                        }
                        (SelectionResult::Opponent(_), PlayerType::Opponent) => {
                            return SelectionResult::Opponent(Selection { index, pillz, fury });
                        }
                        (SelectionResult::Player(_), PlayerType::Player) => {
                            return SelectionResult::Player(Selection { index, pillz, fury });
                        }
                        (SelectionResult::Player(_), PlayerType::Opponent) => {
                            if result.is_none() {
                                result =
                                    Some(SelectionResult::Player(Selection { index, pillz, fury }));
                            }
                        }
                        (SelectionResult::Opponent(_), PlayerType::Player) => {
                            if result.is_none() {
                                result = Some(SelectionResult::Opponent(Selection {
                                    index,
                                    pillz,
                                    fury,
                                }));
                            }
                        }
                    }
                }
            }
        }

        result.unwrap()
    }
}

lazy_static! {
    #[derive(Debug)]
    static ref SHIFT_RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            range.push((n, false));

            if n < 3 {
                for i in 0..n {
                    range.push((i, false));
                }
            } else {
                range.push((n - 3, true));
                range.push((n - 3, false));

                for i in 0..n - 3 {
                    range.push((i, true));
                    range.push((i, false));
                }

                range.push((n - 2, false));
                range.push((n - 1, false));
            }

            ranges.push(range);
        }

        ranges
    };
    static ref SPLIT_SHIFT_RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            range.push((n, false));

            if n < 3 {
                for i in 0..n {
                    range.push((i, false));
                }
            } else {
                range.push((n - 3, false));

                for i in 0..n - 3 {
                    range.push((i, false));
                }

                range.push((n - 2, false));
                range.push((n - 1, false));

                range.push((n - 3, true));
                for i in 0..n - 3 {
                    range.push((i, true));
                }
            }

            ranges.push(range);
        }

        ranges
    };
    static ref SHIFT_FALSE_RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            range.push((n, false));

            if n < 3 {
                for i in 0..n {
                    range.push((i, false));
                }
            } else {
                for i in 0..n - 2 {
                    range.push((i, false));
                }

                range.push((n - 2, false));
                range.push((n - 1, false));
            }

            ranges.push(range);
        }

        ranges
    };
    static ref RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            if n >= 3 {
                for i in 0..n-2 {
                    range.push((i, false));
                    range.push((i, true));
                }

                for i in n-2..=n {
                    range.push((i, false));
                }
            }

            ranges.push(range);
        }

        ranges
    };
    static ref SPLIT_RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            for i in 0..=n {
                range.push((i, false));
            }

            for i in 0..=n-3 {
                range.push((i, true));
            }

            ranges.push(range);
        }

        ranges
    };
    static ref FALSE_RANGES: Vec<Vec<(u8, bool)>> = {
        let mut ranges = Vec::with_capacity(20);
        for n in 0..20u8 {
            let mut range = Vec::with_capacity(n as usize);

            for i in 0..=n {
                range.push((i, false));
            }

            ranges.push(range);
        }

        ranges
    };
}

// #[inline]
// fn shift_range(n: u8) -> Iter<'static, (u8, bool)> {
//     SHIFT_RANGES[n as usize].iter()
// }

#[inline]
fn split_shift_range(n: u8) -> Iter<'static, (u8, bool)> {
    SPLIT_SHIFT_RANGES[n as usize].iter()
}

// #[inline]
// fn range(n: u8) -> Iter<'static, (u8, bool)> {
//     RANGES[n as usize].iter()
// }

#[inline]
fn split_range(n: u8) -> Iter<'static, (u8, bool)> {
    SPLIT_RANGES[n as usize].iter()
}

// #[inline]
// fn false_range(n: u8, round: u8) -> Iter<'static, (u8, bool)> {
//     if round == 0 {
//         FALSE_RANGES[n as usize].iter()
//     } else {
//         SPLIT_RANGES[n as usize].iter()
//     }
// }

#[inline]
fn shift_false_range(n: u8, round: u8) -> Iter<'static, (u8, bool)> {
    if round == 0 {
        SHIFT_FALSE_RANGES[n as usize].iter()
    } else {
        SPLIT_SHIFT_RANGES[n as usize].iter()
    }
}

#[test]
fn test() {
    for i in 0..20 {
        println!("{:#?}", SHIFT_RANGES[i]);
    }
}
