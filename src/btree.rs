use std::fmt::{Debug, Display};
mod dot;

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

    pub fn post_order_iter<'a>(& 'a self) -> PostOrderTraversalIter<'a, T>{
        PostOrderTraversalIter::new(self)
    }

    pub fn pre_order_iter<'a>(& 'a self) -> PreOrderTraversalIter<'a, T>{
        PreOrderTraversalIter::new(self)
    }
}

impl<T: Ord> Tree<T> {
    pub fn insert(&mut self, value: T) {
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

pub struct PostOrderTraversalIter<'a, T> {
    stack: Vec<(Address, &'a Node<T>)>,
}

impl<'a, T> PostOrderTraversalIter<'a, T> {
    fn new(tree: &'a Tree<T>) -> PostOrderTraversalIter<'a, T> {
        match tree.root {
            None => PostOrderTraversalIter { stack: Vec::new() },
            Some(ref node) => PostOrderTraversalIter {
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


impl<'a, T> Iterator for PostOrderTraversalIter<'a, T> {
    type Item = & 'a T;
    fn next(&mut self) -> Option<Self::Item> {
        self.next_item()
    }
}

pub struct PreOrderTraversalIter<'a, T>{
    stack: Vec<TreeStackItem<'a, T>>
}

pub struct TreeItem<'a, T>{
    pub id: usize,
    pub level: usize,
    pub value: & 'a T,
    pub leaf: bool
}

struct TreeStackItem<'a, T>{
    id: usize,
    level: usize,
    node: & 'a Node<T>
}

impl<'a, T> PreOrderTraversalIter<'a, T>{
    fn new(tree: & 'a Tree<T>) -> PreOrderTraversalIter<'a, T>{
        match tree.root {
            None => PreOrderTraversalIter { stack: Vec::new() },
            Some(ref node) => PreOrderTraversalIter {
                stack: vec![TreeStackItem{id: 1, level: 1, node: &node}],
            },
        }
    }
}

impl<'a, T> Iterator for PreOrderTraversalIter<'a, T>{
    type Item = TreeItem<'a, T>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(item) = self.stack.pop() {
            let mut leaf: bool = true;
            if let Some(ref left) = item.node.left{
                self.stack.push(TreeStackItem{id: item.id << 1, level: item.level + 1, node: & left});
                leaf = false;
            }
            if let Some(ref right) = item.node.right{
                self.stack.push(TreeStackItem{id: (item.id << 1) + 1, level: item.level + 1, node: & right});
                leaf = false;
            }
            Some(TreeItem{id: item.id, level: item.level, value: & item.node.value, leaf})
        }else{
            None
        }
    }
}



impl<T: Display> Tree<T>{
    pub fn dot_dump(&self, left: &str, right: &str) -> String{
        let mut graph =  dot::Dot::new();
        let shape = "box";
        let leaf_style = "rounded,filled";
        let leaf_color = "green";
        let leaf_shape = "box";
        for item in self.pre_order_iter(){
            let name = format!("node{}",item.id);
            let parent_name = format!("node{}",item.id >> 1);
            let label = item.value.to_string();
            graph.add_node(
                name.clone(),
                label,
                if item.leaf {leaf_shape.to_owned()} else {shape.to_owned()},
                if item.leaf {Some(leaf_style.to_owned())} else {None},
                if item.leaf {Some(leaf_color.to_owned())} else {None},
            );
            if item.id > 1 {
                let edgelabel : String= if item.id % 2 == 0{
                    left.to_owned()
                }else{
                    right.to_owned()
                };
                graph.add_edge(parent_name, name.clone(),edgelabel);
            }
            graph.append_rank(item.level - 1, name);
        }
        graph.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn post_order() {
        let mut tree: Tree<i64> = Tree::new();
        for num in [6, 1, 0, 2, 5, 4, 9, 8, 3, 7]{
            tree.insert(num);
        }
        println!("{:?}", tree);
        let result: Vec<i64> = tree.post_order_iter().map(|x| (*x).clone()).collect();
        assert_eq!(result, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn pre_order() {
        let mut tree: Tree<i64> = Tree::new();
        for num in [6, 1, 0, 2, 5, 4, 9, 8, 3, 7]{
            tree.insert(num);
        }
        println!("{:?}", tree);
        let result: Vec<i64> = tree.pre_order_iter().map(|x| (*(x.value)).clone()).collect();
        assert_eq!(result, vec![6, 9, 8, 7, 1, 2, 5, 4, 3, 0]);
    }
}
