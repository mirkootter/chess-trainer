pub type DynFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output=T>>>;

pub trait UI {
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<crate::components::board::Arrow>);
    fn shake(&self);
    fn get_user_move(&self) -> DynFuture<shakmaty::Move>;
}

pub async fn train(ui: impl UI) {
    let moves = [
        shakmaty::Move::Normal {
            from: shakmaty::Square::E2,
            to: shakmaty::Square::E4,
            capture: None,
            promotion: None,
            role: shakmaty::Role::Pawn
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::E7,
            to: shakmaty::Square::E5,
            capture: None,
            promotion: None,
            role: shakmaty::Role::Pawn
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::G1,
            to: shakmaty::Square::F3,
            capture: None,
            promotion: None,
            role: shakmaty::Role::Knight
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::G8,
            to: shakmaty::Square::F6,
            capture: None,
            promotion: None,
            role: shakmaty::Role::Knight
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::F3,
            to: shakmaty::Square::E5,
            capture: Some(shakmaty::Role::Pawn),
            promotion: None,
            role: shakmaty::Role::Knight
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::B8,
            to: shakmaty::Square::C6,
            capture: None,
            promotion: None,
            role: shakmaty::Role::Knight
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::E5,
            to: shakmaty::Square::C6,
            capture: Some(shakmaty::Role::Knight),
            promotion: None,
            role: shakmaty::Role::Knight
        },
        shakmaty::Move::Normal {
            from: shakmaty::Square::D7,
            to: shakmaty::Square::C6,
            capture: Some(shakmaty::Role::Knight),
            promotion: None,
            role: shakmaty::Role::Pawn
        },
    ];

    let ui_trainer_move = |m: shakmaty::Move| {
        use crate::components::board::Arrow;

        let arrows = vec![Arrow(m.from().unwrap(), m.to())];
        ui.play_move(m, arrows);
    };

    let mut iter = moves.iter();
    if let Some(trainer_move) = iter.next() {
        ui_trainer_move(trainer_move.clone());
    }

    while let Some(expected) = iter.next() {
        loop {
            let user_move = ui.get_user_move().await;
            if &user_move == expected {
                break;
            } else {
                ui.shake();
            }
        }

        ui.play_move(expected.clone(), Vec::new());

        if let Some(trainer_move) = iter.next() {
            // TODO: Small timeout
            ui_trainer_move(trainer_move.clone());
        } else {
            break;
        }
    }
}
