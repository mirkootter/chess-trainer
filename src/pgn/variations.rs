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

struct Node<'source> {
    parent: Option<std::rc::Rc<Self>>,
    moves: Vec<&'source str>
}

impl<'source> Node<'source> {
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

    pub fn resolve(&self) -> Vec<&'source str> {
        let mut moves = match &self.parent {
            None => Vec::new(),
            Some(node) => node.resolve()
        };

        moves.extend_from_slice(&self.moves);
        moves
    }

    fn parse_internal(self, iter: &mut super::lexer::TokenIterator<'source>, variants: &mut Vec<Self>) {
        let mut main = self;
        while let Some(token) = iter.next() {
            match token {
                super::lexer::Token::StartVariation => {
                    let last_move = main.moves.pop().unwrap(); // This move is not used in the variation
                    let (new_main, variant) = main.fork();
                    variant.parse_internal(iter, variants);
                    main = new_main;
                    main.push(last_move);
                },
                super::lexer::Token::EndVariation => {
                    break;
                },
                super::lexer::Token::SanMove(m) => {
                    main.push(std::str::from_utf8(m).unwrap())
                },
                _ => {}
            }
        }

        variants.push(main);
    }

    pub fn parse(pgn: &'source str) -> Vec<Self> {
        let mut result = Vec::new();
        let root = Self::new();

        let mut iter = super::lexer::TokenIterator::new(pgn.as_bytes());

        root.parse_internal(&mut iter, &mut result);
        result
    }
}

pub struct Variations<'source>(Vec<Node<'source>>);

impl<'source> Variations<'source> {
    pub fn parse(pgn: &'source str) -> Self {
        Variations(Node::parse(pgn))
    }

    pub fn choose(&self, rng: &mut impl rand::Rng) -> Vec<shakmaty::Move> {
        use rand::seq::SliceRandom;
        let variation = self.0.choose(rng).unwrap();

        make_moves(&variation.resolve())
    }
}
