use crate::util::DynFuture;

pub trait UI: Clone {
    fn restart(&self);
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<crate::components::board::Arrow>);
    fn update_arrows(&self, arrows: Vec<crate::components::board::Arrow>);
    fn shake(&self);
    fn get_user_move(&self) -> DynFuture<shakmaty::Move>;
    fn wait_for_restart(&self) -> DynFuture<()>;
    fn wait_for_next_level(&self) -> DynFuture<()>;
    fn show_hints(&self) -> bool;
}

pub async fn train(ui: impl UI + 'static) {
    let pgn = include_str!("../data/stafford.pgn");
    let variants = crate::pgn::movetree::Node::parse(pgn);

    enum UserAction { Restart, NextLevel }

    let wait_for_user_action = || {
        let wait_for_restart = async {
            ui.wait_for_restart().await;
            UserAction::Restart
        };

        let wait_for_next_level = async {
            ui.wait_for_next_level().await;
            UserAction::NextLevel
        };

        futures_micro::or!(wait_for_restart, wait_for_next_level)
    };

    let mut random = rand::thread_rng();

    loop {
        let moves = {
            use rand::seq::SliceRandom;
            let variant = variants.choose(&mut random).unwrap();
            crate::pgn::movetree::make_moves(&variant.resolve())
        };

        loop {
            let training = train_moves(ui.clone(), moves.clone());
            let training = crate::util::spawn_local_cancellable(training);
    
            match wait_for_user_action().await {
                UserAction::NextLevel => { break; },
                UserAction::Restart => { }
            }
    
            training.cancel();
        }
    }
}

pub async fn train_moves(ui: impl UI, moves: Vec<shakmaty::Move>) {
    let ui_trainer_move = |m: shakmaty::Move, hint: Option<shakmaty::Move>| {
        use crate::components::board::Arrow;

        let mut arrows = vec![Arrow(m.from().unwrap(), m.to())];
        if let Some(hint) = hint {
            if ui.show_hints() {
                arrows.push(Arrow(hint.from().unwrap(), hint.to()));
            }
        }
        ui.play_move(m, arrows);
    };

    ui.restart();

    let mut iter = moves.iter();
    if let Some(trainer_move) = iter.next() {
        crate::util::sleep(150).await;
        ui_trainer_move(trainer_move.clone(), iter.clone().next().cloned());
    }

    while let Some(expected) = iter.next() {
        let mut errors = 0;
        loop {
            let user_move = ui.get_user_move().await;
            if &user_move == expected {
                break;
            } else {
                ui.shake();
                errors = errors + 1;

                if errors == 3 {
                    // wait a small delay for the shake to end
                    crate::util::sleep(300).await;

                    use crate::components::board::Arrow;
                    ui.update_arrows(vec![Arrow(expected.from().unwrap(), expected.to())]);
                }
            }
        }

        ui.play_move(expected.clone(), Vec::new());

        if let Some(trainer_move) = iter.next() {
            crate::util::sleep(150).await;
            ui_trainer_move(trainer_move.clone(), iter.clone().next().cloned());
        } else {
            break;
        }
    }
}
