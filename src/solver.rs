use std::{
    slice::Iter,
    thread,
    time::{Duration, Instant},
};

use colored::Colorize;
use lazy_static::lazy_static;

use crate::{
    ability, battle,
    game::{self, Game, GameStatus, PlayerType, Selection},
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

impl Solver {
    pub fn solve(game: &Game) -> SelectionResult {
        let solve_count: u64;
        unsafe {
            ability::PRINT = false;
            game::PRINT = false;
            modifiers::PRINT = false;
            battle::PRINT = false;
            solve_count = SOLVE_COUNT;
        }
        let handler = thread::spawn(|| unsafe {
            loop {
                let solve_count = SOLVE_COUNT;

                for _ in 0..10 {
                    let solve_count = SOLVE_COUNT;

                    thread::sleep(Duration::from_millis(500));

                    if solve_count == SOLVE_COUNT {
                        return;
                    }
                }

                println!(
                    "{} {} /5s",
                    " Solve Count ".white().on_magenta(),
                    SOLVE_COUNT - solve_count
                );
            }
        });
        let now = Instant::now();
        let best = if game.s1.is_none() != game.s2.is_none() {
            Solver::solve_second(game)
        } else {
            Solver::solve_first(game)
        };
        unsafe {
            ability::PRINT = true;
            game::PRINT = true;
            modifiers::PRINT = true;
            battle::PRINT = true;

            let solves = SOLVE_COUNT - solve_count;
            let elapsed = now.elapsed();
            println!(
                "{} {} /{:.1?}secs ({:.0?}k/s) - Final count",
                " Solve Count ".white().on_magenta(),
                solves,
                elapsed.as_secs_f32(),
                solves as f32 / elapsed.as_secs_f32() / 1000f32
            );
        }
        handler.join().unwrap();
        best
    }

    pub fn solve_second(game: &Game) -> SelectionResult {
        // {
        //     let status = game.status();
        //     if status != GameStatus::Playing {
        //         match status {
        //             GameStatus::Draw => return Sele
        //         }
        //     }
        // }
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

            for &(pillz, fury) in shift_range(pillz1) {
                let mut worst = GameResult::Win;
                for &(p, f) in shift_range(pillz2) {
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

            for &(pillz, fury) in shift_range(game.get_turn_player().pillz) {
                let mut g = game.clone();

                let battled = g.select(index, pillz, fury);

                let status = g.status();
                if battled && status != GameStatus::Playing {
                    match (g.status(), turn) {
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
}

#[inline]
fn shift_range(n: u8) -> Iter<'static, (u8, bool)> {
    SHIFT_RANGES[n as usize].iter()
}

#[test]
fn test() {
    for i in 0..20 {
        println!("{:?}", SHIFT_RANGES[i]);
    }
}
