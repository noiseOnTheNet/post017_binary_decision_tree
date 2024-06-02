use std::fs;
use decision::btree::Tree;

fn main(){
    let mut tree: Tree<i64> = Tree::new();
    tree.insert(8);
    tree.insert(10);
    tree.insert(4);
    tree.insert(6);
    tree.insert(5);
    let graph = tree.dot_dump(">", "<");
    fs::write("./example.dot", graph).expect("Unable to write file example.dot");
}
