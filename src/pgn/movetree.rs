use super::tree;
use super::lexer::{Token, TokenIterator};
use std::rc::Rc;

type Node<'source> = tree::Node<&'source str>;
type Tree<'source> = tree::Tree<&'source str>;

pub struct MoveTree<'source>(Tree<'source>);

pub struct Variation<'source, 'tree> {
    tree: &'tree MoveTree<'source>,
    node: Rc<Node<'source>>
}


pub struct Variations<'source, 'tree> {
    tree: &'tree MoveTree<'source>,
    variations: Vec<Rc<Node<'source>>>
}

impl<'source> MoveTree<'source> {
    pub fn new() -> Self {
        MoveTree(Tree::new())
    }

    pub fn add_pgn(&mut self, pgn: &'source str) {
        let mut iter = TokenIterator::new(pgn.as_bytes());

        let root = self.0.root.clone();
        self.parse_internal(root, &mut iter);
    }

    fn parse_internal(&mut self, node: Rc<Node<'source>>, iter: &mut TokenIterator<'source>) {
        let mut main = node;
        let mut start_variation = false;
        while let Some(token) = iter.next() {
            match token {
                Token::SanMove(m) => {
                    let m = std::str::from_utf8(m).unwrap();

                    if start_variation {
                        start_variation = false;
                        let fork = main.fork_or_find(&mut self.0, m);
                        self.parse_internal(fork, iter);
                    } else {
                        main = main.branch_or_find(&mut self.0, m);
                    }
                },
                Token::StartVariation => {
                    assert!(!start_variation);
                    start_variation = true;
                },
                Token::EndVariation => break,
                _ => {}
            }
        }
    }

    pub fn get_all_variations<'tree>(&'tree self) -> Variations<'source, 'tree> {
        let mut variations = Vec::new();
        self.get_all_variations_from_node(&self.0.root, &mut variations);

        Variations {
            tree: self,
            variations
        }
    }

    fn get_all_variations_from_node(&self, node: &Rc<Node<'source>>, result: &mut Vec<Rc<Node<'source>>>) {
        let children = node.get_children(&self.0);
        if children.is_empty() {
            result.push(node.clone());
            return;

        }
        for child in children {
            self.get_all_variations_from_node(child, result);
        }
    }

    fn resolve_variation_internal(&self, node: &Node<'source>, result: &mut Vec<&'source str>) {
        if let Some(parent) = node.try_get_parent(&self.0) {
            self.resolve_variation_internal(&parent, result);
            result.push(node.value(&self.0).unwrap());
        }
    }
}

impl<'source, 'tree> Variation<'source, 'tree> {
    pub fn resolve(&self) -> Vec<&'source str> {
        let count = {
            let mut count = 0;
            let mut node = self.node.clone();

            while let Some(parent) = node.try_get_parent(&self.tree.0) {
                count = count + 1;
                node = parent;
            }

            count
        };

        let mut result = Vec::new();
        result.reserve_exact(count);

        self.tree.resolve_variation_internal(&self.node, &mut result);

        result
    }

    pub fn make_moves(&self) -> Vec<shakmaty::Move> {
        let moves = self.resolve();
        let mut result = Vec::new();
        result.reserve_exact(moves.len());

        let mut pos = shakmaty::Chess::default();
        for m in moves {
            let san: shakmaty::san::San = m.parse().unwrap();
            let m = san.to_move(&pos).unwrap();
    
            use shakmaty::Position;
            pos = pos.play(&m).unwrap();
    
            result.push(m);    
        }

        result
    }
}

impl<'source, 'tree> Variations<'source, 'tree> {
    pub fn choose(&self, rng: &mut impl rand::Rng) -> Variation<'source, 'tree> {
        use rand::seq::SliceRandom;
        Variation {
            tree: self.tree,
            node: self.variations.choose(rng).unwrap().clone()
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.variations.len()
    }

    pub fn get(&self, index: usize) -> Variation<'source, 'tree> {
        Variation {
            tree: self.tree,
            node: self.variations[index].clone()
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_branch_test() {
        let mut tree = super::MoveTree::new();
        tree.add_pgn("e4 e5 Nf3");
        {
            let variations = tree.get_all_variations();
            assert_eq!(variations.len(), 1);

            assert_eq!(variations.get(0).resolve(), ["e4", "e5", "Nf3"]);
        }

        tree.add_pgn("e4 e5 Nc3 Nf6");
        {
            let variations = tree.get_all_variations();
            assert_eq!(variations.len(), 2);

            assert_eq!(variations.get(0).resolve(), ["e4", "e5", "Nf3"]);
            assert_eq!(variations.get(1).resolve(), ["e4", "e5", "Nc3", "Nf6"]);
        }

        tree.add_pgn("e4 e5 Nf3 Nf6 Nxe5");
        {
            let variations = tree.get_all_variations();
            assert_eq!(variations.len(), 2);

            assert_eq!(variations.get(0).resolve(), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations.get(1).resolve(), ["e4", "e5", "Nc3", "Nf6"]);
        }

        tree.add_pgn("d4 d5 c4");
        {
            let variations = tree.get_all_variations();
            assert_eq!(variations.len(), 3);

            assert_eq!(variations.get(0).resolve(), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations.get(1).resolve(), ["e4", "e5", "Nc3", "Nf6"]);
            assert_eq!(variations.get(2).resolve(), ["d4", "d5", "c4"]);
        }
    }

    #[test]
    fn pgn_with_variations() {
        let mut tree = super::MoveTree::new();
        tree.add_pgn("e4 (d4 d5 c4) e5 Nf3 (Nc3 Nf6) Nf6 Nxe5");
        {
            let variations = tree.get_all_variations();
            assert_eq!(variations.len(), 3);

            assert_eq!(variations.get(0).resolve(), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations.get(1).resolve(), ["e4", "e5", "Nc3", "Nf6"]);
            assert_eq!(variations.get(2).resolve(), ["d4", "d5", "c4"]);
        }
    }
}