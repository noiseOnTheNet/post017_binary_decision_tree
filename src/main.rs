use decision::{evaluate_best_split, DTreeBuilder};
use polars::prelude::*;
use std::collections::HashSet;
mod btree;

fn main() -> polars::prelude::PolarsResult<()> {
    let features = HashSet::from(["sepal_length", "sepal_width", "petal_length", "petal_width"]);
    let target = "variety";

    let data = load_data("iris.csv", target)?;

    let rule = evaluate_best_split(& data, & features, target)?;

    println!(
        "\nrule\n{1:->0$}{2:?}{1:-<0$}\n",
        20, "\n", rule
    );

    let builder = DTreeBuilder::new(features, target)
        .set_max_level(7);

    let tree = builder.build(& data)?;

    decision::print_tree(& tree);

    println!("{}",& tree.dot_dump());
    Ok(())
}

fn load_data(path: &str, target: &str) -> PolarsResult<DataFrame> {
    // read data file
    let mut data = CsvReader::from_path(path)?.has_header(true).finish()?;
    println!("\ndata\n{1:->0$}{2:?}{1:-<0$}\n", 20, "\n", data);

    // set target column as categorical
    data.try_apply(target, |s| {
        s.cast(&DataType::Categorical(None, CategoricalOrdering::Lexical))
    })?;
    println!("\ndata\n{1:->0$}{2:?}{1:-<0$}\n", 20, "\n", data);

    Ok(data)
}
