use std::fmt::Debug;

#[derive(Debug)]
pub struct Node<T> {
    pub value: T,
    pub left: Option<Box<Node<T>>>,
    pub right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    pub fn new(value: T) -> Node<T> {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

impl<T> From<Node<T>> for Option<Box<Node<T>>> {
    fn from(value: Node<T>) -> Self {
        Some(Box::new(value))
    }
}

#[derive(Debug, Default)]
pub struct Tree<T> {
    root: Option<Box<Node<T>>>,
}

impl<T> Tree<T> {
    pub fn new() -> Tree<T> {
        Tree { root: None }
    }

    pub fn from_node(node: Node<T>) -> Tree<T> {
        Tree {
            root: Some(node.into()),
        }
    }

    pub fn depth_iter<'a>(& 'a self) -> DepthTraversalIter<'a, T>{
        DepthTraversalIter::new(self)
    }

    pub fn breadth_iter<'a>(& 'a self) -> BreadthTraversalIter<'a, T>{
        BreadthTraversalIter::new(self)
    }
}

impl<T: Ord> Tree<T> {
    fn insert(&mut self, value: T) {
        match self.root {
            None => {
                self.root = Node::new(value).into();
            }
            Some(ref mut node) => {
                Tree::<T>::insert_recursive(node, value);
            }
        }
    }

    fn insert_recursive(node: &mut Node<T>, value: T) {
        if value > node.value {
            match node.right {
                None => {
                    node.right = Node::new(value).into();
                }
                Some(ref mut n) => {
                    Tree::<T>::insert_recursive(n, value);
                }
            }
        } else if value < node.value {
            match node.left {
                None => {
                    node.left = Node::new(value).into();
                }
                Some(ref mut n) => {
                    Tree::<T>::insert_recursive(n, value);
                }
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum Address {
    Enter,
    LeftCompleted,
    ValueYield,
    Completed,
}

pub struct DepthTraversalIter<'a, T> {
    stack: Vec<(Address, &'a Node<T>)>,
}

impl<'a, T> DepthTraversalIter<'a, T> {
    fn new(tree: &'a Tree<T>) -> DepthTraversalIter<'a, T> {
        match tree.root {
            None => DepthTraversalIter { stack: Vec::new() },
            Some(ref node) => DepthTraversalIter {
                stack: vec![(Address::Enter, &node)],
            },
        }
    }
    fn next_item(&mut self) -> Option<&'a T> {
        while let Some((address, node)) = self.stack.pop() {
            match address {
                Address::Enter => match node.left {
                    None => {
                        self.stack.push((Address::LeftCompleted, node));
                    }
                    Some(ref left) => {
                        self.stack.push((Address::LeftCompleted, node));
                        self.stack.push((Address::Enter, left));
                    }
                },
                Address::LeftCompleted => {
                    self.stack.push((Address::ValueYield, node));
                    return Some(&node.value);
                }
                Address::ValueYield => match node.right {
                    None => {
                        self.stack.push((Address::Completed, node));
                    }
                    Some(ref right) => {
                        self.stack.push((Address::Completed, node));
                        self.stack.push((Address::Enter, right));
                    }
                },
                Address::Completed => {}
            }
        }
        None
    }
}


impl<'a, T> Iterator for DepthTraversalIter<'a, T> {
    type Item = & 'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_item()
    }
}

pub struct BreadthTraversalIter<'a, T>{
    stack: Vec<(usize, & 'a Node<T>)>
}

impl<'a, T> BreadthTraversalIter<'a, T>{
    fn new(tree: & 'a Tree<T>) -> BreadthTraversalIter<'a, T>{
        match tree.root {
            None => BreadthTraversalIter { stack: Vec::new() },
            Some(ref node) => BreadthTraversalIter {
                stack: vec![(0, &node)],
            },
        }
    }
}

impl<'a, T> Iterator for BreadthTraversalIter<'a, T>{
    type Item = (usize, & 'a T);
    fn next(&mut self) -> Option<Self::Item> {
        if let Some((level, node)) = self.stack.pop() {
            if let Some(ref left) = node.left{
                self.stack.push((level + 1, & left));
            }
            if let Some(ref right) = node.right{
                self.stack.push((level + 1, & right));
            }
            Some((level, & node.value))
        }else{
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_a_root_node() {
        let mut tree: Tree<i64> = Tree::new();
        tree.insert(8);
        tree.insert(10);
        tree.insert(4);
        tree.insert(6);
        tree.insert(5);
        println!("{:?}", tree);
        let result: Vec<i64> = tree.depth_iter().map(|x| (*x).clone()).collect();
        assert_eq!(result, vec![4, 5, 6, 8, 10]);
    }
}
