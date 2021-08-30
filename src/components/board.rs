#[derive(Clone)]
pub struct Arrow(pub shakmaty::Square, pub shakmaty::Square);

#[derive(yew::Properties, Clone)]
pub struct BoardProps {
    pub board: std::rc::Rc<shakmaty::Chess>,
    pub arrows: Vec<Arrow>,
    pub on_user_move: std::rc::Rc<dyn Fn(shakmaty::Move) -> BoardAction>
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BoardAction {
    None,
    Shake
}

pub struct Board {
    link: yew::ComponentLink<Self>,
    board: std::rc::Rc<shakmaty::Chess>,
    selected: Option<shakmaty::Square>,
    arrows: Vec<Arrow>,
    container_ref: yew::NodeRef,
    on_user_move: std::rc::Rc<dyn Fn(shakmaty::Move) -> BoardAction>
}

fn shake(node_ref: &yew::NodeRef) {
    if let Some(elem) = node_ref.cast::<web_sys::HtmlElement>() {
        elem.set_class_name("");
        elem.offset_height();
        elem.set_class_name("shake-animation");
    }
}

const COL_CLASSES: [&'static str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const ROW_CLASSES: [&'static str; 8] = [" row1", " row2", " row3", " row4", " row5", " row6", " row7", " row8"];

fn get_class_for_piece(piece: shakmaty::Piece, square: shakmaty::Square) -> String {
    let mut result = String::new();
    result.reserve(19); // "bishop black a row8" longest possible classname

    match piece.role {
        shakmaty::Role::Bishop => result += "bishop",
        shakmaty::Role::King => result += "king",
        shakmaty::Role::Knight => result += "knight",
        shakmaty::Role::Pawn => result += "pawn",
        shakmaty::Role::Rook => result += "rook",
        shakmaty::Role::Queen => result += "queen"
    }

    match piece.color {
        shakmaty::Color::Black => result += " black ",
        shakmaty::Color::White => result += " white ",
    }

    result += COL_CLASSES[square.file() as usize];
    result += ROW_CLASSES[square.rank() as usize];

    result
}

fn get_class_for_position(square: shakmaty::Square) -> String {
    let mut result = String::new();
    result.reserve(5);

    result += COL_CLASSES[square.file() as usize];
    result += ROW_CLASSES[square.rank() as usize];

    result
}

#[derive(Debug)]
pub enum BoardMessage {
    SelectPiece(shakmaty::Square)
}

impl yew::Component for Board {
    type Message = BoardMessage;
    type Properties = BoardProps;

    fn create(props: Self::Properties, link: yew::ComponentLink<Self>) -> Self {
        Self {
            link,
            board: props.board,
            selected: None,
            arrows: props.arrows,
            container_ref: yew::NodeRef::default(),
            on_user_move: props.on_user_move
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            BoardMessage::SelectPiece(selected) => {
                let changed = self.selected != Some(selected);
                self.selected = Some(selected);
                changed
            }
        }
    }

    fn change(&mut self, props: Self::Properties) -> yew::ShouldRender {
        self.board = props.board;
        self.arrows = props.arrows;

        self.selected = None;
        true // TODO: Some check maybe?
    }

    fn view(&self) -> yew::Html {
        let moves = if self.selected.is_some() {
            use shakmaty::Position;
            let legals = self.board.legal_moves();
            Some(legals.into_iter().filter(|m| m.from() == self.selected))
        } else {
            None
        };

        let arrow_polygon = |w: f32,h: f32| {
            use std::fmt::Write;

            let wh = w / 2.0f32;
            let head_w = 1.5f32 * w;
            let head = h - 1.5f32 * head_w;
            let mut result = String::new();
            write!(result, "{},{} ", wh, 0).unwrap();
            write!(result, "{},{} ", wh, head).unwrap();
            write!(result, "{},{} ", head_w, head).unwrap();
            write!(result, "{},{} ", 0, h).unwrap();
            write!(result, "{},{} ", -head_w, head).unwrap();
            write!(result, "{},{} ", -wh, head).unwrap();
            write!(result, "{},{}", -wh, 0.0f32).unwrap();

            result
        };

        let make_arrow = |from: shakmaty::Square, to: shakmaty::Square| {
            let x0 = from.file() as u8 as f32;
            let y0 = 7.0 - from.rank() as u8 as f32;
            let x1 = to.file() as u8 as f32;
            let y1 = 7.0 - to.rank() as u8 as f32;

            let dx = x1 - x0;
            let dy = y1 - y0;

            let length = (dx * dx + dy * dy).sqrt();

            let angle = dy.atan2(dx).to_degrees() - 90.0f32;
            let transform = format!("translate({},{}) rotate({})", x0, y0, angle);

            yew::html_nested! {
                <polygon transform=transform points=arrow_polygon(0.2,length) class="red" stroke="none" />   
            }
        };

        yew::html! {
            <cb-container ref=self.container_ref.clone()>
                <cb-board/>
                {
                    {
                        use shakmaty::Setup;
                        self.board.board().pieces().map(|(square, piece)| {
                            let class_name = get_class_for_piece(piece, square);

                            yew::html! {
                                <cb-piece class={class_name} onmousedown=self.link.callback(move |_|
                                    BoardMessage::SelectPiece(square)
                                ) />
                            }
                        }).collect::<yew::Html>()
                    }
                }
                {
                    if let Some(selected) = self.selected {
                        yew::html! {
                            <cb-square-marker class={get_class_for_position(selected)} />
                        }
                    } else {
                        yew::html! {}
                    }
                }
                {
                    if let Some(moves) = moves {
                        moves.map(|m| {
                            let mut class_name = get_class_for_position(m.to());
                            if m.is_capture() {
                                class_name += " capture";
                            }

                            let node_ref = self.container_ref.clone();
                            let on_user_move = self.on_user_move.clone();
                            let on_user_move: yew::Callback<yew::MouseEvent> = (move |_| {
                                match on_user_move(m.clone()) {
                                    BoardAction::None => {},
                                    BoardAction::Shake => shake(&node_ref)
                                }
                            }).into();

                            yew::html! {
                                <cb-point-marker class={class_name}
                                    onmousedown=on_user_move />
                            }
                        }).collect::<yew::Html>()
                    } else {
                        yew::html!{}
                    }
                }
                <svg viewBox="-0.5 -0.5 8 8">
                    {
                        self.arrows.iter().map(|Arrow(from, to)| { make_arrow(*from, *to) }).collect::<yew::Html>()
                    }
                </svg>
            </cb-container>
        }
    }
}