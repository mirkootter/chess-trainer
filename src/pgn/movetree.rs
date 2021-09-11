use super::tree;
use super::lexer::{Token, TokenIterator};
use std::rc::Rc;

pub struct MoveTree<'source>(tree::Tree<&'source str>);

pub struct Variation<'source>(Rc<tree::Node<&'source str>>);
pub struct Variations<'source, 'tree> {
    tree: &'tree MoveTree<'source>,
    variations: Vec<Variation<'source>>
}

impl<'source> MoveTree<'source> {
    pub fn new() -> Self {
        MoveTree(tree::Tree::new())
    }

    pub fn add_pgn(&mut self, pgn: &'source str) {
        let mut iter = TokenIterator::new(pgn.as_bytes());

        let root = self.0.root.clone();
        self.parse_internal(root, &mut iter);
    }

    fn parse_internal(&mut self, node: Rc<tree::Node<&'source str>>, iter: &mut TokenIterator<'source>) {
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

    fn get_all_variations_from_node(&self, node: &Rc<tree::Node<&'source str>>, result: &mut Vec<Variation<'source>>) {
        let children = node.get_children(&self.0);
        if children.is_empty() {
            result.push(Variation(node.clone()));
            return;

        }
        for child in children {
            self.get_all_variations_from_node(child, result);
        }
    }

    fn resolve_variation_internal(&self, node: &tree::Node<&'source str>, result: &mut Vec<&'source str>) {
        if let Some(parent) = node.try_get_parent(&self.0) {
            self.resolve_variation_internal(&parent, result);
            result.push(node.value(&self.0).unwrap());
        }
    }
}

impl<'source> Variation<'source> {
    pub fn resolve(&self, tree: &MoveTree<'source>) -> Vec<&'source str> {
        let count = {
            let mut count = 0;
            let mut node = self.0.clone();

            while let Some(parent) = node.try_get_parent(&tree.0) {
                count = count + 1;
                node = parent;
            }

            count
        };

        let mut result = Vec::new();
        result.reserve_exact(count);

        tree.resolve_variation_internal(&self.0, &mut result);

        result
    }

    pub fn make_moves(&self, tree: &MoveTree<'source>) -> Vec<shakmaty::Move> {
        let moves = self.resolve(tree);
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

impl<'source> Variations<'source, '_> {
    pub fn choose(&self, rng: &mut impl rand::Rng) -> Vec<shakmaty::Move> {
        use rand::seq::SliceRandom;
        let variation = self.variations.choose(rng).unwrap();

        variation.make_moves(self.tree)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn simple_branch_test() {
        let mut tree = super::MoveTree::new();
        tree.add_pgn("e4 e5 Nf3");
        {
            let variations = tree.get_all_variations().variations;
            assert_eq!(variations.len(), 1);

            assert_eq!(variations[0].resolve(&tree), ["e4", "e5", "Nf3"]);
        }

        tree.add_pgn("e4 e5 Nc3 Nf6");
        {
            let variations = tree.get_all_variations().variations;
            assert_eq!(variations.len(), 2);

            assert_eq!(variations[0].resolve(&tree), ["e4", "e5", "Nf3"]);
            assert_eq!(variations[1].resolve(&tree), ["e4", "e5", "Nc3", "Nf6"]);
        }

        tree.add_pgn("e4 e5 Nf3 Nf6 Nxe5");
        {
            let variations = tree.get_all_variations().variations;
            assert_eq!(variations.len(), 2);

            assert_eq!(variations[0].resolve(&tree), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations[1].resolve(&tree), ["e4", "e5", "Nc3", "Nf6"]);
        }

        tree.add_pgn("d4 d5 c4");
        {
            let variations = tree.get_all_variations().variations;
            assert_eq!(variations.len(), 3);

            assert_eq!(variations[0].resolve(&tree), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations[1].resolve(&tree), ["e4", "e5", "Nc3", "Nf6"]);
            assert_eq!(variations[2].resolve(&tree), ["d4", "d5", "c4"]);
        }
    }

    #[test]
    fn pgn_with_variations() {
        let mut tree = super::MoveTree::new();
        tree.add_pgn("e4 (d4 d5 c4) e5 Nf3 (Nc3 Nf6) Nf6 Nxe5");
        {
            let variations = tree.get_all_variations().variations;
            assert_eq!(variations.len(), 3);

            assert_eq!(variations[0].resolve(&tree), ["e4", "e5", "Nf3", "Nf6", "Nxe5"]);
            assert_eq!(variations[1].resolve(&tree), ["e4", "e5", "Nc3", "Nf6"]);
            assert_eq!(variations[2].resolve(&tree), ["d4", "d5", "c4"]);
        }
    }
}