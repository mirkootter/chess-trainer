use yew::prelude::*;
mod components;
mod pgn_lexer;
mod trainer;
mod util;

enum GameMessage {
    PlayMove(shakmaty::Move, Vec<components::board::Arrow>),
    UpdateArrows(Vec<components::board::Arrow>),
    RegisterMoveSender(async_oneshot::Sender<shakmaty::Move>),
    SetLearning(bool)
}

struct Game {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    board_link_ref: components::board::LinkRef,
    board: std::rc::Rc<shakmaty::Chess>,
    arrows: Vec<components::board::Arrow>,
    move_sender: Option<std::cell::RefCell<async_oneshot::Sender<shakmaty::Move>>>,
    learning_input_ref: yew::NodeRef,
    learning: bool
}

impl Game {
    fn send_user_move_internal(&self, m: shakmaty::Move) {
        if let Some(sender) = &self.move_sender {
            let _ = sender.borrow_mut().send(m);
        }
    }
}

impl Component for Game {
    type Message = GameMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link: link,
            board_link_ref: Default::default(),
            board: std::rc::Rc::new(shakmaty::Chess::default()),
            arrows: Vec::new(),
            move_sender: None,
            learning_input_ref: Default::default(),
            learning: true
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GameMessage::PlayMove(m, arrows) => {
                use shakmaty::Position;
                let mut board = (*self.board).clone();
                board.play_unchecked(&m);
                self.board = board.into();
                self.arrows = arrows;
                
                true
            },
            GameMessage::UpdateArrows(arrows) => {
                self.arrows = arrows;
                true
            },
            GameMessage::RegisterMoveSender(sender) => {
                self.move_sender = Some(sender.into());
                false
            },
            GameMessage::SetLearning(learning) => {
                self.learning = learning;
                true
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            wasm_bindgen_futures::spawn_local(trainer::train(UI {
                link: self.link.clone(),
                board_link_ref: self.board_link_ref.clone()
            }))
        }
    }

    fn view(&self) -> Html {
        let link = self.link.clone();
        let on_user_move: yew::Callback<shakmaty::Move> = (move |m: shakmaty::Move| {
            if let Some(game) = link.get_component() {
                game.send_user_move_internal(m);
            }
        }).into();

        let learning_ref = self.learning_input_ref.clone();
        let on_learning_change = self.link.callback(move |_| {
            if let Some(input) = learning_ref.cast::<web_sys::HtmlInputElement>() {
                GameMessage::SetLearning(!input.checked())
            } else {
                GameMessage::SetLearning(true)
            }
        });

        html! {
            <div class="game">
                <components::board::Board
                    board=self.board.clone()
                    arrows=self.arrows.clone()
                    on_user_move=on_user_move
                    link_ref=self.board_link_ref.clone()
                    reverse=true />
                <div>
                    <label class="switch">
                        <input type="checkbox" ref=self.learning_input_ref.clone() checked=!self.learning onclick=on_learning_change />
                        <div />
                    </label>
                    {if self.learning {"Lernmodus (Pfeile anzeigen)"} else {"Übungsmodus (ohne Pfeile)"}}
                </div>
            </div>
        }
    }
}

struct UI {
    link: ComponentLink<Game>,
    board_link_ref: components::board::LinkRef
}

impl trainer::UI for UI {
    fn play_move(&self, m: shakmaty::Move, arrows: Vec<components::board::Arrow>) {
        self.link.send_message(GameMessage::PlayMove(m, arrows));
    }

    fn update_arrows(&self, arrows: Vec<components::board::Arrow>) {
        self.link.send_message(GameMessage::UpdateArrows(arrows))
    }

    fn shake(&self) {
        if let Some(ref board_link) = *self.board_link_ref.borrow() {
            if let Some(comp) = board_link.get_component() {
                comp.shake();
            }
        }
    }

    fn get_user_move(&self) -> util::DynFuture<shakmaty::Move> {
        let (sender, receiver) = util::oneshot();
        self.link.send_message(GameMessage::RegisterMoveSender(sender));
        receiver
    }

    fn show_hints(&self) -> bool {
        if let Some(game) = self.link.get_component() {
            return game.learning;
        }

        true
    }
}

fn main() {
    yew::start_app::<Game>();
}