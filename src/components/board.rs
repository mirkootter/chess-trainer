use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct Arrow(pub shakmaty::Square, pub shakmaty::Square);

impl From<&shakmaty::Move> for Arrow {
    fn from(m: &shakmaty::Move) -> Self {
        Arrow(m.from().unwrap(), m.to())
    }
}

#[derive(Props, PartialEq)]
pub struct BoardProps {
    pub board: by_address::ByAddress<std::rc::Rc<shakmaty::Chess>>,
    pub arrows: Vec<Arrow>,
    pub cb_user_move: crate::util::Sender<shakmaty::Move>,
    pub reverse: bool
}

mod dioxus_elements {
    pub use dioxus::prelude::dioxus_elements::*;

    #[allow(non_camel_case_types)]
    pub struct cbcontainer;
    
    #[allow(non_camel_case_types)]
    pub struct cbboard;

    #[allow(non_camel_case_types)]
    pub struct cbpiece;

    #[allow(non_camel_case_types)]
    pub struct cbsquaremarker;

    #[allow(non_camel_case_types)]
    pub struct cbpointmarker;

    impl cbcontainer {
        pub const TAG_NAME: &'static str = "cb-container";
        pub const NAME_SPACE: Option<&'static str> = None;
    }

    impl cbboard {
        pub const TAG_NAME: &'static str = "cb-board";
        pub const NAME_SPACE: Option<&'static str> = None;
    }

    impl cbpiece {
        pub const TAG_NAME: &'static str = "cb-piece";
        pub const NAME_SPACE: Option<&'static str> = None;
    }
    impl GlobalAttributes for cbpiece {}

    impl cbsquaremarker {
        pub const TAG_NAME: &'static str = "cb-square-marker";
        pub const NAME_SPACE: Option<&'static str> = None;
    }
    impl GlobalAttributes for cbsquaremarker {}

    impl cbpointmarker {
        pub const TAG_NAME: &'static str = "cb-point-marker";
        pub const NAME_SPACE: Option<&'static str> = None;
    }
    impl GlobalAttributes for cbpointmarker {}
}

const COL_CLASSES: [&'static str; 8] = ["a", "b", "c", "d", "e", "f", "g", "h"];
const ROW_CLASSES: [&'static str; 8] = [" row1", " row2", " row3", " row4", " row5", " row6", " row7", " row8"];

fn get_class_for_piece(piece: shakmaty::Piece, square: shakmaty::Square, reverse: bool) -> String {
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

    let col = if reverse { 7 - square.file() as usize } else { square.file() as usize };
    let row = if reverse { 7 - square.rank() as usize } else { square.rank() as usize };

    result += COL_CLASSES[col];
    result += ROW_CLASSES[row];

    result
}

fn get_class_for_position(square: shakmaty::Square, reverse: bool) -> String {
    let mut result = String::new();
    result.reserve(5);

    let col = if reverse { 7 - square.file() as usize } else { square.file() as usize };
    let row = if reverse { 7 - square.rank() as usize } else { square.rank() as usize };

    result += COL_CLASSES[col];
    result += ROW_CLASSES[row];

    result
}

pub fn Board(cx: Scope<BoardProps>) -> Element {
    let selected = use_state::<Option<shakmaty::Square>>(cx, || None);

    let moves = if selected.is_some() {
        use shakmaty::Position;
        let legals = cx.props.board.legal_moves();
        Some(legals.into_iter().filter(|m| m.from() == *selected.get()))
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

    let reverse = cx.props.reverse;

    let make_arrow = |from: shakmaty::Square, to: shakmaty::Square| {
        let mut x0 = from.file() as u8 as f32;
        let mut y0 = 7.0 - from.rank() as u8 as f32;
        let x1 = to.file() as u8 as f32;
        let y1 = 7.0 - to.rank() as u8 as f32;

        let dx = x1 - x0;
        let dy = y1 - y0;

        let length = (dx * dx + dy * dy).sqrt();

        let mut angle = dy.atan2(dx).to_degrees() - 90.0f32;
        if reverse {
            angle += 180.0;
            x0 = 7.0 - x0;
            y0 = 7.0 - y0;
        }
        let transform = format!("translate({},{}) rotate({})", x0, y0, angle);

        rsx! {
            polygon {
                transform: "{transform}",
                points: "{arrow_polygon(0.2,length)}",
                class: "red",
                stroke: "none"
            }
        }
    };


    cx.render(rsx! {
        cbcontainer {
            cbboard {}
            {
                use shakmaty::Setup;
                cx.props.board.board().pieces().map(|(square, piece)| {
                    let class_name = get_class_for_piece(piece, square, reverse);

                    rsx! {
                        cbpiece {
                            class: "{class_name}",
                            onmousedown: move |_| { selected.set(Some(square)); }
                        }
                    }
                })
            }
            if let Some(selected) = *selected.get() {
                rsx! {
                    cbsquaremarker {
                        class: "{get_class_for_position(selected, reverse)}"
                    }
                }
            }
            if let Some(moves) = moves {
                rsx! {
                    moves.map(|m| {
                        let mut class_name = get_class_for_position(m.to(), reverse);
                        if m.is_capture() {
                            class_name += " capture";
                        }

                        rsx! {
                            cbpointmarker {
                                class: "{get_class_for_position(m.to(), reverse)}",
                                onmousedown: move |_| {
                                    cx.props.cb_user_move.send(m.clone());
                                }
                            }
                        }
                    })
                }
            }

            svg {
                view_box: "-0.5 -0.5 8 8",
                {
                    cx.props.arrows.iter().map(|Arrow(from, to)| { make_arrow(*from, *to) })
                }
            }
        }
    })
}