use super::tree;
use super::lexer::{Token, TokenIterator};
use std::rc::Rc;

type Node<'source> = tree::Node<&'source str>;
type Tree<'source> = tree::Tree<&'source str>;

pub struct MoveTree<'source>(Tree<'source>);

pub struct Variation<'source, 'tree> {
    tree: &'tree MoveTree<'source>,
    nodes: Vec<Rc<Node<'source>>>
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

    fn resolve_variation_internal(&self, node: Rc<Node<'source>>, result: &'_ mut Vec<Rc<Node<'source>>>) {
        if let Some(parent) = node.try_get_parent(&self.0) {
            self.resolve_variation_internal(parent, result);
            result.push(node);
        }
    }
}

impl<'source, 'tree> Variation<'source, 'tree> {
    fn new(tree: &'tree MoveTree<'source>, target_node: Rc<Node<'source>>) -> Self {
        let count = {
            let mut count = 0;
            let mut node = target_node.clone();

            while let Some(parent) = node.try_get_parent(&tree.0) {
                count = count + 1;
                node = parent;
            }

            count
        };

        let mut nodes = Vec::new();
        nodes.reserve_exact(count);

        tree.resolve_variation_internal(target_node, &mut nodes);

        Variation {
            tree,
            nodes
        }
    }

    pub fn resolve(&self) -> Vec<&'source str> {
        self.nodes.iter().map(|node| node.value(&self.tree.0).unwrap()).collect()
    }

    pub fn iter<'a>(&'a self) -> VariationIterator<'source, 'a> {
        VariationIterator {
            variation: self,
            node_iter: self.nodes.iter(),
            pos: Default::default()
        }
    }
}

impl<'source, 'tree> Variations<'source, 'tree> {
    pub fn choose(&self, rng: &mut impl rand::Rng) -> Variation<'source, 'tree> {
        use rand::seq::SliceRandom;
        Variation::new(self.tree, self.variations.choose(rng).unwrap().clone())
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.variations.len()
    }

    pub fn get(&self, index: usize) -> Variation<'source, 'tree> {
        Variation::new(self.tree, self.variations[index].clone())
    }
}

pub struct VariationIterator<'source, 'a> {
    variation: &'a Variation<'source, 'a>,
    node_iter: std::slice::Iter<'a, Rc<Node<'source>>>,
    pos: shakmaty::Chess
}

impl<'source, 'a> Iterator for VariationIterator<'source, 'a> {
    type Item = shakmaty::Move;

    fn next(&mut self) -> Option<Self::Item> {
        match self.node_iter.next() {
            None => None,
            Some(node) => {
                let m = node.value(&self.variation.tree.0).unwrap();
                let san: shakmaty::san::San = m.parse().unwrap();
                let m = san.to_move(&self.pos).unwrap();
    
                use shakmaty::Position;
                self.pos.play_unchecked(&m);

                Some(m)
            }
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