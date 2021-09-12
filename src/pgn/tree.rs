use std::rc::{Rc, Weak};
use qcell::{QCell, QCellOwner};

pub struct Tree<T> {
    owner: QCellOwner,
    pub root: Rc<Node<T>>
}

impl<T> Tree<T> {
    pub fn new() -> Self {
        let owner = Default::default();
        let root = Node::make_root(&owner);
        Tree {
            owner,
            root
        }
    }
}

pub struct Node<T> {
    inner: QCell<NodeInner<T>>
}

struct NodeInner<T> {
    parent: Option<Weak<Node<T>>>,
    value: Option<T>,
    children: Vec<Rc<Node<T>>>
}

impl<T> Node<T> {
    fn make_root(owner: &QCellOwner) -> Rc<Self> {
        let inner = NodeInner {
            parent: None,
            value: None,
            children: Default::default()
        };
        
        Rc::new(Node {
            inner: QCell::new(owner, inner)
        })
    }

    pub fn branch(self: &Rc<Self>, tree: &mut Tree<T>, value: T) -> Rc<Self> {
        let inner = NodeInner {
            parent: Some(Rc::downgrade(self)),
            value: Some(value),
            children: Default::default()
        };

        let new_node = Rc::new(Node { inner: QCell::new(&mut tree.owner, inner )});
        self.inner.rw(&mut tree.owner).children.push(new_node.clone());

        new_node
    }

    pub fn try_get_parent(&self, tree: &Tree<T>) -> Option<Rc<Self>> {
        match &self.inner.ro(&tree.owner).parent {
            None => None,
            Some(parent) => parent.upgrade()
        }
    }

    fn get_or_create_parent(self: &Rc<Self>, tree: &mut Tree<T>) -> Rc<Self> {
        if let Some(parent) = &self.inner.ro(&tree.owner).parent {
            return parent.upgrade().unwrap();
        }

        let parent = Self::make_root(&tree.owner);
        self.inner.rw(&mut tree.owner).parent = Some(Rc::downgrade(&parent));
        parent.inner.rw(&mut tree.owner).children.push(self.clone());

        tree.root = parent.clone();
        parent
    }

    pub fn _fork(self: &Rc<Self>, tree: &mut Tree<T>, value: T) -> Rc<Self> {
        let parent = self.get_or_create_parent(tree);
        parent.branch(tree, value)
    }

    pub fn get_children<'a>(&'a self, tree: &'a Tree<T>) -> &'a Vec<Rc<Self>> {
        &self.inner.ro(&tree.owner).children
    }

    pub fn find_first_leaf<'a>(self: &'a Rc<Self>, tree: &'a Tree<T>) -> &'a Rc<Self> {
        let mut node = self;
        loop {
            let inner = node.inner.ro(&tree.owner);
            match inner.children.first() {
                None => return node,
                Some(child) => { node = child; }
            }
        }
    }
}

impl<T: PartialEq> Node<T> {
    pub fn branch_or_find(self: &Rc<Self>, tree: &mut Tree<T>, value: T) -> Rc<Self> {
        let inner = self.inner.ro(&tree.owner);
        for child in &inner.children {
            if child.inner.ro(&tree.owner).value.as_ref() == Some(&value) {
                return child.clone();
            }
        }

        self.branch(tree, value)
    }

    pub fn fork_or_find(self: &Rc<Self>, tree: &mut Tree<T>, value: T) -> Rc<Self> {
        if self.inner.ro(&tree.owner).value.as_ref() == Some(&value) {
            return self.clone();
        }

        let parent = self.get_or_create_parent(tree);
        let parent_inner = parent.inner.ro(&tree.owner);
        for child in &parent_inner.children {
            if child.inner.ro(&tree.owner).value.as_ref() == Some(&value) {
                return child.clone();
            }
        }

        parent.branch(tree, value)
    }
}

impl<T: Copy> Node<T> {
    pub fn value(&self, tree: &Tree<T>) -> Option<T> {
        self.inner.ro(&tree.owner).value
    }
}
