use std::fs;
use decision::btree::Tree;

fn main(){
    let mut tree: Tree<i64> = Tree::new();
    for num in [6, 1, 0, 2, 5, 4, 9, 8, 3, 7]{
        tree.insert(num);
    }
    let graph = tree.dot_dump("<", ">");
    fs::write("./example.dot", graph).expect("Unable to write file example.dot");
}
