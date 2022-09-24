use std::sync::Mutex;

use actix_cors::Cors;
use actix_web::{post, web::Json, App, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use serde::Deserialize;

use crate::{
    card::Hand,
    game::{Game, GameStatus, PlayerType, Selection},
    solver::{SelectionResult, Solver},
};

lazy_static! {
    static ref GAME: Mutex<Option<Game>> = Mutex::new(None);
}

pub async fn serve() -> Result<(), std::io::Error> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            // .app_data(Data::new(Mutex::<Option<Game>>::new(None)))
            .service(input)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

#[post("/")]
async fn input(data: Json<Input>) -> impl Responder {
    println!("data -> {:?}", data);
    // let mut game = state.lock().unwrap();
    let mut game = GAME.lock().unwrap();
    match data.0 {
        Input::Game {
            cards,
            flip,
            life,
            pillz,
        } => {
            let h1 = Hand::from_names(
                cards[0].as_str(),
                cards[1].as_str(),
                cards[2].as_str(),
                cards[3].as_str(),
            );
            let h2 = Hand::from_names(
                cards[4].as_str(),
                cards[5].as_str(),
                cards[6].as_str(),
                cards[7].as_str(),
            );

            let mut g = Game::new(h1, h2);
            g.flip = flip;
            g.p1.life = life;
            g.p2.life = life;
            g.p1.pillz = pillz;
            g.p2.pillz = pillz;
            *game = Some(g.clone());

            g.print_status();

            if flip == 0 {
                // actix_web::web::block(move || {
                //     println!("Blocking setup");
                Solver::middle(&g);
                //     println!("Unblocking setup");
                // })
                // .await
                // .expect("Error in setup block");
            }

            println!("{} turn", g.get_turn_name());
        }
        // Input::Cancel { cancel: _ } => {
        //     if let Some(game) = game.as_mut() {
        //         game.clear_selection();
        //         game.print_status();
        //     }
        // }
        Input::Selection(Selection { index, pillz, fury }) => {
            if let Some(game) = game.as_mut() {
                select(game, index, pillz, fury, false).await;
            } else {
                println!("{:?}", game);
            }
        }
        Input::CancelSelection {
            cancel: _,
            selection: Selection { index, pillz, fury },
        } => {
            if let Some(game) = game.as_mut() {
                game.clear_selection();
                select(game, index, pillz, fury, true).await;
            } else {
                println!("{:?}", game);
            }
        }
    }

    HttpResponse::Ok()
}

async fn select(game: &mut Game, index: usize, pillz: u8, fury: bool, cancelled: bool) {
    if !game.can_select(index, pillz, fury) {
        return;
    }

    println!("Select {} {} {}", index, pillz, fury);
    let battled = game.select(index, pillz, fury);
    if !battled {
        game.print_status();
    }
    if game.status() != GameStatus::Playing {
        return;
    }

    let turn = game.get_turn();

    let g = game.clone();
    // actix_web::web::block(move || {
    //     println!("Blocking");
    if g.round == 0 {
        if !cancelled && turn == PlayerType::Player {
            Solver::middle(&g);
        }
    } else {
        let best = Solver::solve(&g);

        match (best, turn) {
            (SelectionResult::Player(_), PlayerType::Opponent)
            | (SelectionResult::Opponent(_), PlayerType::Player) => {
                Solver::middle(&g);
            }
            (_, _) => println!("{:?}", best),
        }
    }
    //     println!("Unblocking");
    // })
    // .await
    // .expect("Error in setup block");

    println!("{} turn", game.get_turn_name());
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Input {
    Game {
        cards: Vec<String>,
        #[serde(default)]
        flip: u8,
        #[serde(default = "default_12")]
        life: u8,
        #[serde(default = "default_12")]
        pillz: u8,
    },
    Selection(Selection),
    // Cancel {
    //     cancel: bool,
    // },
    CancelSelection {
        cancel: bool,
        selection: Selection,
    },
}

fn default_12() -> u8 {
    12
}
