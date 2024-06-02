use std::fmt;
use std::fmt::Display;
use core::default::Default;
use std::mem::{take, replace};

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

#[derive(Default)]
struct DotRank(Vec<String>);
impl DotRank{
    fn new() -> DotRank{
        DotRank(
            Vec::new()
        )
    }
}

impl Display for DotRank{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{{rank = same; {};}}",self.0.join("; "))
    }
}

pub struct Dot{
    nodes: Vec<DotNode>,
    edges: Vec<DotEdge>,
    ranks: Vec<DotRank>
}

impl Dot{
    pub fn new() -> Self{
        Dot{
            nodes: Vec::new(),
            edges: Vec::new(),
            ranks: Vec::new(),
        }
    }

    pub fn add_node(& mut self, name: String, label: String, shape: String, style: Option<String>, fillcolor: Option<String>) -> () {
        let node = DotNode{name, label, shape, style, fillcolor};
        self.nodes.push(node);
    }

    pub fn add_edge(& mut self, first: String, second: String, label: String) -> () {
        let node = DotEdge{first, second, label};
        self.edges.push(node);
    }

    pub fn append_rank(& mut self, index: usize, node: String) -> (){
        //ensure space
        while self.ranks.len() <= index {
            self.ranks.push(DotRank::new())
        }
        let mut bin = take(& mut self.ranks[index]);
        bin.0.push(node);
        let _ = replace(& mut self.ranks[index], bin);
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
