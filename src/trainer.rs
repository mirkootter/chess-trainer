pub type DynFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output=T>>>;

pub trait UI {
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<crate::components::board::Arrow>);
    fn shake(&self);
    fn get_user_move(&self) -> DynFuture<shakmaty::Move>;
}

fn make_moves(san_moves: &[&'static str]) -> Vec<shakmaty::Move> {
    let mut result = Vec::new();
    result.reserve_exact(san_moves.len());

    let mut pos = shakmaty::Chess::default();
    for m in san_moves {
        let san: shakmaty::san::San = m.parse().unwrap();
        let m = san.to_move(&pos).unwrap();

        use shakmaty::Position;
        pos = pos.play(&m).unwrap();

        result.push(m);
    }

    result
}

pub async fn train(ui: impl UI) {
    let moves = make_moves(&[
        "e4", "e5",
        "Nf3", "Nf6",
        "Nxe5", "Nc6",
        "Nxc6", "dxc6",
        "d3", "Bc5",
        "Bg5", "Nxe4",
        "Bxd8", "Bxf2",
        "Ke2", "Bg4"
    ]);

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
