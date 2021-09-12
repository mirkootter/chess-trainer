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
    let mut movetree = crate::pgn::movetree::MoveTree::new();
    movetree.add_pgn(include_str!("../data/stafford.pgn"));
    //movetree.add_pgn(include_str!("../data/kid.pgn"));

    let variations = std::rc::Rc::new(movetree).get_all_variations();

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
    let mut variation = variations.choose(&mut random);

    loop {
        let training = train_moves(ui.clone(), variation.clone());
        let training = crate::util::spawn_local_cancellable(training);

        match wait_for_user_action().await {
            UserAction::NextLevel => { variation = variations.choose(&mut random); },
            UserAction::Restart => { }
        }

        training.cancel();
    }
}

pub async fn train_moves(ui: impl UI, variation: crate::pgn::movetree::Variation<'static>) {
    let ui_trainer_move = |m: shakmaty::Move, hint: Option<shakmaty::Move>| {
        let mut arrows = vec![(&m).into()];
        if let Some(hint) = hint {
            if ui.show_hints() {
                arrows.push((&hint).into());
            }
        }
        ui.play_move(m, arrows);
    };

    ui.restart();

    let mut iter = variation.iter();

    let mut current_player = Player::Trainer; // Trainer plays white and makes the first move

    while let Some(expected_move) = iter.next() {
        match current_player {
            Player::Trainer => {
                crate::util::sleep(150).await;
                ui_trainer_move(expected_move, iter.peek());
            },
            Player::Student => {
                let mut errors = 0;
                loop {
                    let user_move = ui.get_user_move().await;
                    if user_move == expected_move {
                        break;
                    } else {
                        ui.shake();
                        errors = errors + 1;
        
                        if errors == 3 {
                            // wait a small delay for the shake to end
                            crate::util::sleep(300).await;
                            ui.update_arrows(vec![(&expected_move).into()]);
                        }
                    }
                }
        
                ui.play_move(expected_move, Vec::new());
            }
        }

        current_player = current_player.next();
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player { Student, Trainer }

impl Player {
    fn next(self) -> Self {
        match self {
            Self::Student => Self::Trainer,
            Self::Trainer => Self::Student
        }
    }
}