use yew::prelude::*;
mod components;
mod pgn;
mod trainer;
mod util;

enum GameMessage {
    Restart,
    PlayMove(shakmaty::Move, Vec<components::board::Arrow>),
    UpdateArrows(Vec<components::board::Arrow>),
    SetLearning(bool)
}

struct Game {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    board_link_ref: components::board::LinkRef,
    board: std::rc::Rc<shakmaty::Chess>,
    arrows: Vec<components::board::Arrow>,
    user_move_channel: util::EventChannel<shakmaty::Move>,
    user_action_channel: util::EventChannel<trainer::UserAction>,
    learning_input_ref: yew::NodeRef,
    learning: bool
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
            user_move_channel: util::EventChannel::new(),
            user_action_channel: util::EventChannel::new(),
            learning_input_ref: Default::default(),
            learning: true
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GameMessage::Restart => {
                self.board = Default::default();
                self.arrows = Vec::new();
                true
            },
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
                <div class="game-header">
                    <label class="switch">
                        <input type="checkbox" ref=self.learning_input_ref.clone() checked=!self.learning onclick=on_learning_change />
                        <div />
                    </label>
                    {if self.learning {"Lernmodus (Pfeile anzeigen)"} else {"Übungsmodus (ohne Pfeile)"}}
                </div>
                <components::board::Board
                    board=self.board.clone()
                    arrows=self.arrows.clone()
                    on_user_move=self.user_move_channel.callback()
                    link_ref=self.board_link_ref.clone()
                    reverse=true />
                <div class="desktop-flex-break" />
                <div class="game-footer">
                    <components::iconbutton::IconButton
                        disabled=false
                        image="images/icons/refresh_black_24dp.svg"
                        onclick=self.user_action_channel.callback_constant(trainer::UserAction::Restart) />
                    <components::iconbutton::IconButton
                        disabled=false
                        image="images/icons/double_arrow_black_24dp.svg"
                        onclick=self.user_action_channel.callback_constant(trainer::UserAction::NextLevel) />
                </div>
            </div>
        }
    }
}

#[derive(Clone)]
struct UI {
    link: ComponentLink<Game>,
    board_link_ref: components::board::LinkRef
}

impl trainer::UI for UI {
    fn restart(&self) {
        self.link.send_message(GameMessage::Restart);
    }

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
        match self.link.get_component() {
            Some(game) => {
                game.user_move_channel.receive()
            },
            None => Box::pin(std::future::pending())
        }
    }

    fn wait_for_user_action(&self) -> util::DynFuture<trainer::UserAction> {
        match self.link.get_component() {
            Some(game) => {
                game.user_action_channel.receive()
            },
            None => Box::pin(std::future::pending())
        }
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
