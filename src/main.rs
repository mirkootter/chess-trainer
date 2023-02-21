#![allow(non_snake_case)]

mod components;
mod pgn;
mod trainer;
mod util;

use dioxus::prelude::*;
use futures_util::StreamExt;

fn main() {
    // launch the web app
    dioxus_web::launch(App);
}

// create a component that renders a div with the text "Hello, world!"
fn App(cx: Scope) -> Element {
    let arrows = [
        components::board::Arrow(shakmaty::Square::D7, shakmaty::Square::D5),
        components::board::Arrow(shakmaty::Square::E7, shakmaty::Square::E5)
    ];

    let chess = std::rc::Rc::new(shakmaty::Chess::default());

    let cb_user_move = use_coroutine(cx, |mut rx| async move {
        
        while let Some(m) = rx.next().await {
            // TODO
            gloo_console::log!(format!("Move {:?}", m));
        }
    });

    cx.render(rsx! {
        components::board::Board {
            board: chess.into(),
            arrows: arrows.into(),
            cb_user_move: util::Sender(cb_user_move.clone()),
            reverse: false
        }
    })
}
