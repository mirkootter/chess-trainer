use yew::prelude::*;
mod components;
mod trainer;

enum GameMessage {
    PlayMove(shakmaty::Move),
    RegisterMoveSender(async_oneshot::Sender<shakmaty::Move>)
}

struct Game {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    board_link_ref: components::board::LinkRef,
    board: std::rc::Rc<shakmaty::Chess>,
    arrows: Vec<components::board::Arrow>,
    move_sender: Option<std::cell::RefCell<async_oneshot::Sender<shakmaty::Move>>>
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
        use shakmaty::Square;
        use components::board::Arrow;
        Self {
            link: link,
            board_link_ref: Default::default(),
            board: std::rc::Rc::new(shakmaty::Chess::default()),
            arrows: Vec::new(),
            move_sender: None
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            GameMessage::PlayMove(m) => {
                use shakmaty::Position;
                let mut board = (*self.board).clone();
                board.play_unchecked(&m);
                self.board = board.into();
                
                true
            },
            GameMessage::RegisterMoveSender(sender) => {
                self.move_sender = Some(sender.into());
                false
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

        html! {
            <components::board::Board
                board=self.board.clone()
                arrows=self.arrows.clone()
                on_user_move=on_user_move
                link_ref=self.board_link_ref.clone() />
        }
    }
}

struct UI {
    link: ComponentLink<Game>,
    board_link_ref: components::board::LinkRef
}

impl trainer::UI for UI {
    fn show_arrows(&mut self, _arrows: Vec<crate::components::board::Arrow>) {
        // TODO
    }

    fn play_move(&mut self, m: shakmaty::Move) {
        self.link.send_message(GameMessage::PlayMove(m));
    }

    fn shake(&mut self) {
        if let Some(ref board_link) = *self.board_link_ref.borrow() {
            if let Some(comp) = board_link.get_component() {
                comp.shake();
            }
        }
    }

    fn get_user_move(&mut self) -> trainer::DynFuture<shakmaty::Move> {
        let (sender, receiver) = async_oneshot::oneshot();

        self.link.send_message(GameMessage::RegisterMoveSender(sender));

        Box::pin(async move {
            receiver.await.unwrap()
        })
    }
}

fn main() {
    yew::start_app::<Game>();
}