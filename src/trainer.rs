use crate::util::DynFuture;

pub trait UI: Clone {
    fn init(&self, pos: &shakmaty::Chess, explore: bool);
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<crate::components::board::Arrow>);
    fn update_arrows(&self, arrows: Vec<crate::components::board::Arrow>);
    fn shake(&self);
    fn get_user_move(&self) -> DynFuture<shakmaty::Move>;
    fn wait_for_user_action(&self) -> DynFuture<UserAction>;
    fn show_hints(&self) -> bool;
}

#[derive(Clone)]
pub enum UserAction {
    Restart,
    NextLevel,
    ToggleExplore
}

#[derive(Clone)]
struct SharedGame(std::rc::Rc<std::cell::RefCell<GameInner>>);

struct GameInner {
    iter: crate::pgn::movetree::VariationIterator<'static>,
    last_turn: Player,
    explore: bool
}

impl GameInner {
    fn new(variation: &crate::pgn::movetree::Variation<'static>) -> Self {
        GameInner {
            iter: variation.iter(),
            last_turn: Player::Trainer, // We want to the trainer to play,
            explore: false
        }
    }
}

impl SharedGame {
    pub fn new(variation: &crate::pgn::movetree::Variation<'static>) -> Self {
        SharedGame(std::cell::RefCell::new(GameInner::new(variation)).into())
    }

    pub fn current_player(&self) -> Player {
        self.0.borrow().last_turn
    }

    pub fn is_explore(&self) -> bool {
        self.0.borrow().explore
    }

    pub fn reset(&self) {
        let mut inner = self.0.borrow_mut();
        inner.iter.reset();
        inner.last_turn = Player::Trainer;
    }

    pub fn start_variation(&self, variation: &crate::pgn::movetree::Variation<'static>) {
        let mut inner = self.0.borrow_mut();
        inner.iter = variation.iter();
        inner.last_turn = Player::Trainer;
    }

    pub fn next(&self) -> Option<shakmaty::Move> {
        let mut inner = self.0.borrow_mut();
        inner.last_turn = inner.last_turn.next();
        inner.iter.next()
    }

    pub fn peek(&self) -> Option<shakmaty::Move> {
        self.0.borrow().iter.peek()
    }

    pub fn peek_all(&self) -> Vec<shakmaty::Move> {
        self.0.borrow().iter.peek_all()
    }

    pub fn try_switch(&self, m: &shakmaty::Move) -> bool {
        self.0.borrow_mut().iter.try_switch(m)
    }

    pub fn toggle_explore(&self) {
        let mut inner = self.0.borrow_mut();
        inner.explore = !inner.explore;
    }

    pub fn position(&self) -> std::cell::Ref<shakmaty::Chess> {
        std::cell::Ref::map(self.0.borrow(), |inner| inner.iter.position())
    }
}

pub async fn train(ui: impl UI + 'static) {
    let mut movetree = crate::pgn::movetree::MoveTree::new();
    movetree.add_pgn(include_str!("../data/stafford.pgn"));
    //movetree.add_pgn(include_str!("../data/kid.pgn"));

    let variations = std::rc::Rc::new(movetree).get_all_variations();

    let mut random = rand::thread_rng();
    let game = SharedGame::new(&variations.choose(&mut random));

    loop {
        let training = train_moves(ui.clone(), game.clone());
        let training = crate::util::spawn_local_cancellable(training);

        match ui.wait_for_user_action().await {
            UserAction::NextLevel => { game.start_variation(&variations.choose(&mut random)); },
            UserAction::Restart => { game.reset(); },
            UserAction::ToggleExplore => { game.toggle_explore(); }
        }

        training.cancel();
    }
}

async fn train_moves(ui: impl UI, game: SharedGame) {
    let explore = game.is_explore(); // TODO

    let ui_trainer_move = |m: shakmaty::Move, hint: Option<shakmaty::Move>| {
        let mut arrows = vec![(&m).into()];
        if let Some(hint) = hint {
            if ui.show_hints() {
                arrows.push((&hint).into());
            }
        }
        ui.play_move(m, arrows);
    };

    ui.init(&game.position(), game.is_explore());

    loop {
        if explore {
            let arrows = game.peek_all().iter().map::<crate::components::board::Arrow, _>(|m| m.into()).collect();
            ui.update_arrows(arrows);

            loop {
                let user_move = ui.get_user_move().await;
                if game.try_switch(&user_move) {
                    break;
                } else {
                    ui.shake();
                }
            }

            if let Some(next_move) = game.next() {
                ui.play_move(next_move, Vec::new());
            }
        } else {
            let expected_move = match game.peek() {
                Some(m) => m,
                None => break
            };

            match game.current_player() {
                Player::Trainer => {
                    crate::util::sleep(150).await;

                    let expected_move = game.next().unwrap();
                    ui_trainer_move(expected_move, game.peek());
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
            
                    let expected_move = game.next().unwrap();
                    ui.play_move(expected_move, Vec::new());
                }
            }
        }
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