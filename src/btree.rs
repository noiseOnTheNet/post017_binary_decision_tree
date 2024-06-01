use std::fmt;
use std::fmt::{Debug, Display};
use std::mem::take;

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

impl<'a, T> BreadthTraversalIter<'a, T>{
    fn new(tree: & 'a Tree<T>) -> BreadthTraversalIter<'a, T>{
        match tree.root {
            None => BreadthTraversalIter { stack: Vec::new() },
            Some(ref node) => BreadthTraversalIter {
                stack: vec![TreeStackItem{id: 1, level: 1, node: &node}],
            },
        }
    }
}

impl<'a, T> Iterator for BreadthTraversalIter<'a, T>{
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


struct DotNode{
    name: String,
    label: String,
    shape: String,
    style: Option<String>,
    fillcolor: Option<String>
}

impl Display for DotNode{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let style: String = match self.style {
            None => "".into(),
            Some(ref kind) => format!(", style=\"{}\"",kind)
        };
        let fillcolor: String = match self.fillcolor {
            None => "".into(),
            Some(ref kind) => format!(", fillcolor=\"{}\"",kind)
        };
        write!(f,"{} [label=\"{}\", shape=\"{}\"{}{}];",self.name, self.label, self.shape, style, fillcolor)
    }
}

struct DotEdge{
    first: String,
    second: String,
    label: String
}

impl Display for DotEdge{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} -> {} [label=\"{}\"]",self.first,self.second,self.label)
    }
}

struct DotRank(Vec<String>);

impl Display for DotRank{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{{rank = same; {};}}",self.0.join("; "))
    }
}

struct Dot{
    nodes: Vec<DotNode>,
    edges: Vec<DotEdge>,
    ranks: Vec<DotRank>
}

impl Dot{
    fn new() -> Self{
        Dot{
            nodes: Vec::new(),
            edges: Vec::new(),
            ranks: Vec::new(),
        }
    }

    fn add_node(& mut self, name: String, label: String, shape: String, style: Option<String>, fillcolor: Option<String>) -> () {
        let node = DotNode{name, label, shape, style, fillcolor};
        self.nodes.push(node);
    }

    fn add_edge(& mut self, first: String, second: String, label: String) -> () {
        let node = DotEdge{first, second, label};
        self.edges.push(node);
    }
}

impl Display for Dot{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut graph: Vec<String>=vec!["digraph {".into(),"rankdir = BT;".into(),"subgraph{".into()];
        for node in &self.nodes{
            graph.push(node.to_string());
        }
        for edge in &self.edges{
            graph.push(edge.to_string());
        }
        for rank in &self.ranks{
            graph.push(rank.to_string());
        }
        graph.push("}".into());
        graph.push("}".into());
        write!(f,"{}",graph.join("\n"))
    }
}

impl<T: Display> Tree<T>{
    pub fn dot_dump(&self, left: &str, right: &str) -> String{
        let mut graph =  Dot::new();
        let mut ranks: Vec<DotRank>= Vec::new();
        let shape = "box";
        let leaf_style = "rounded,filled";
        let leaf_color = "green";
        let leaf_shape = "box";
        for item in self.breadth_iter(){
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
            if ranks.len() < item.level{
                ranks.push(DotRank(Vec::new()));
            }
            let mut bin = ranks.get(item.level - 1).unwrap().0.clone();
            bin.push(name);
            let _  = std::mem::replace(& mut ranks[item.level - 1], DotRank(bin));
        }
        graph.ranks = ranks;
        graph.to_string()
    }
}

// digraph {
//    rankdir = BT;
//    subgraph {
//         root [label="(4)", shape="box"];
//         child1 [label="(6)", shape="box"];
//         child2 [label="(2)", shape="box"];
//         granchild1 [label="(3)", shape="box"];
//         granchild2 [label="(1)", shape="box"];
//         granchild3 [label="(8)", shape="box"];
//         granchild4 [label="(5)", shape="box"];
//         root -> child1
//         root -> child2
//         child2 -> granchild1
//         child2 -> granchild2
//         child1 -> granchild3
//         child1 -> granchild4
//         {rank = same; child1; child2;}
//         {rank = same; granchild1; granchild2; granchild3; granchild4;}
//    }
// }
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
