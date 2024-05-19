use decision::{evaluate_metric, predict_majority_dataframe, evaluate_best_split};
use polars::prelude::*;
use std::collections::HashSet;

fn main() -> polars::prelude::PolarsResult<()> {
    let features = HashSet::from(["sepal_length", "sepal_width", "petal_length", "petal_width"]);
    let target = "variety";

    let data = load_data("iris.csv", target)?;

    let rule = evaluate_best_split(& data, & features, target)?;

    println!(
        "\nrule\n{1:->0$}{2:?}{1:-<0$}\n",
        20, "\n", rule
    );
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
