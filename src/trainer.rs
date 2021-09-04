use crate::util::DynFuture;

pub trait UI {
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<crate::components::board::Arrow>);
    fn update_arrows(&self, arrows: Vec<crate::components::board::Arrow>);
    fn shake(&self);
    fn get_user_move(&self) -> DynFuture<shakmaty::Move>;
    fn show_hints(&self) -> bool;
}

fn make_moves(san_moves: &[&'_ str]) -> Vec<shakmaty::Move> {
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

struct MoveTreeNode<'source> {
    parent: Option<std::rc::Rc<Self>>,
    moves: Vec<&'source str>
}

impl<'source> MoveTreeNode<'source> {
    fn new() -> Self {
        (Self {
            parent: None,
            moves: Vec::new()
        }).into()
    }

    fn push(&mut self, m: &'source str) {
        self.moves.push(m);
    }

    fn fork(self) -> (Self, Self) {
        let this: std::rc::Rc<Self> = self.into();
        let a = Self {
            parent: Some(this.clone()),
            moves: Vec::new()
        };
        let b = Self {
            parent: Some(this),
            moves: Vec::new()
        };

        (a, b)
    }

    fn resolve(&self) -> Vec<&'source str> {
        let mut moves = match &self.parent {
            None => Vec::new(),
            Some(node) => node.resolve()
        };

        moves.extend_from_slice(&self.moves);
        moves
    }

    fn parse_internal(self, iter: &mut crate::pgn_lexer::TokenIterator<'source>, variants: &mut Vec<Self>) {
        let mut main = self;
        while let Some(token) = iter.next() {
            match token {
                crate::pgn_lexer::Token::StartVariation => {
                    let last_move = main.moves.pop().unwrap(); // This move is not used in the variation
                    let (new_main, variant) = main.fork();
                    variant.parse_internal(iter, variants);
                    main = new_main;
                    main.push(last_move);
                },
                crate::pgn_lexer::Token::EndVariation => {
                    break;
                },
                crate::pgn_lexer::Token::SanMove(m) => {
                    main.push(std::str::from_utf8(m).unwrap())
                },
                _ => {}
            }
        }

        variants.push(main);
    }

    fn parse(pgn: &'source str) -> Vec<Self> {
        let mut result = Vec::new();
        let root = Self::new();

        let mut iter = crate::pgn_lexer::TokenIterator::new(pgn.as_bytes());

        root.parse_internal(&mut iter, &mut result);
        result
    }
}

pub async fn train(ui: impl UI) {
    let pgn = include_str!("../data/stafford.pgn");
    let variants = MoveTreeNode::parse(pgn);
    let moves = {
        use rand::seq::SliceRandom;
        let variant = variants.choose(&mut rand::thread_rng()).unwrap();
        
        make_moves(&variant.resolve())
    };
    
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

    let mut iter = moves.iter();
    if let Some(trainer_move) = iter.next() {
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
