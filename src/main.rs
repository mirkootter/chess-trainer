use yew::prelude::*;
mod components;

enum GameMessage {
    PlayMove(shakmaty::Move)
}

struct Game {
    // `ComponentLink` is like a reference to a component.
    // It can be used to send messages to the component
    link: ComponentLink<Self>,
    board: std::rc::Rc<shakmaty::Chess>,
    arrows: Vec<components::board::Arrow>
}

impl Component for Game {
    type Message = GameMessage;
    type Properties = ();

    fn create(_props: Self::Properties, link: ComponentLink<Self>) -> Self {
        use shakmaty::Square;
        use components::board::Arrow;
        Self {
            link: link,
            board: std::rc::Rc::new(shakmaty::Chess::default()),
            arrows: vec![Arrow(Square::E2, Square::E4), Arrow(Square::B1, Square::C3)]
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
            }
        }
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        false
    }

    fn view(&self) -> Html {
        let link = self.link.clone();
        let on_user_move: std::rc::Rc<dyn Fn(shakmaty::Move) -> components::board::BoardAction> = std::rc::Rc::new(move |m| {
            if m.from() == Some(shakmaty::Square::E2) && m.to() == shakmaty::Square::E4 {
                link.send_message(GameMessage::PlayMove(m));
                components::board::BoardAction::None
            } else {
                components::board::BoardAction::Shake
            }
        });

        html! {
            <components::board::Board board=self.board.clone() arrows=self.arrows.clone() on_user_move=on_user_move />
        }
    }
}

fn main() {
    yew::start_app::<Game>();
}