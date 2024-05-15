mod btree;
use polars::lazy::dsl::Expr;
use polars::prelude::*;
use polars::series::Series;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Rule {
    dimension: String,
    cutoff: f64,
    metric: f64,
}

// categorical types are mapped to u32 because:
// 1. do not are equivalent to rust enums which are actually sum types
// 2. we target also 64bit execution platforms like webasm
#[derive(Debug)]
pub struct Decision {
    rule: Option<Rule>,
    confidence: f64,
    prediction: String,
}

pub struct DTreeBuilder {
    max_level: usize,
    min_size: usize,
}

// uses a struct to define trees constraints
impl DTreeBuilder {
    fn build_node(
        &self,
        data: & DataFrame,
        level: usize,
        features: HashSet<&str>,
        target: &str,
    ) -> PolarsResult<btree::Node<Decision>> {
        let mut prediction = predict_majority_dataframe(data, target)?;
        let mut node = btree::Node::new(prediction);
        // check stop conditions
        if (prediction.confidence < 1.0) && // all elements belong to one category
            (data.shape().0 > self.min_size) && // size is below minimum threshold
            (level <= self.max_level){ // maximum depth reached
                let rule = evaluate_best_split(data, features, target)?;
                let higher = data
                    .clone()
                    .lazy()
                    .filter(col(& rule.dimension).gt(rule.cutoff))
                    .collect();
                let lower = data
                    .clone()
                    .lazy()
                    .filter(col(& rule.dimension).gt_eq(rule.cutoff))
                    .collect();
                prediction.rule = Some(rule);
                node.left = self.build_node(higher, level + 1, features, target).into();
                node.right = self.build_node(lower, level + 1, features, target).into();
            }
        Ok(node)
    }
    pub fn build(
        &self,
        data: & DataFrame,
        features: HashSet<&str>,
        target: &str,
    ) -> PolarsResult<btree::Tree<Decision>> {
        let root = self.build_node(data, 1, features, target)?;
        Ok(btree::Tree::from_node(root))
    }
}

// Gini impurity metric
pub fn estimate_gini(data: &DataFrame, target: &str) -> PolarsResult<f64> {
    let label_count: DataFrame = data.column(target)?.categorical()?.value_counts()?;

    let expr: Expr = (col("counts").cast(DataType::Float64) / col("counts").sum())
        .pow(2)
        .alias("squares");

    let squared: DataFrame = label_count.lazy().select([expr]).collect()?;

    let square_sum: f64 = squared.column("squares")?.sum()?;

    Ok(1.0 - square_sum)
}

// returns the name of the majority category
pub fn predict_majority_dataframe<'a>(data: &'a DataFrame, target: &str) -> PolarsResult<Decision> {
    // extract the categorical target column
    let labels = data.column(target)?.categorical()?;

    let total = labels.len() as f64;

    // count all categories and sort them
    let result_count = labels.value_counts()?;
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", result_count);

    // get the most frequent category
    let result_cat = result_count.head(Some(1));
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", result_cat);

    // transform the series into a categorical vector
    let actual_cat = result_cat
        .column(target)?
        .categorical()?;

    // collect all categories as strings
    let string_cat: Vec<String> = actual_cat
        .iter_str()
        .flatten()
        .map(|name| (*name).into())
        .collect();
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", string_cat);

    let probability: Vec<f64>= result_cat
        .column("counts")?
        .f64()?
    .iter()
    .flatten()
    .map(|c| c/total)
        .collect();
    // return the most common category as a string
    return Ok(
        Decision{
            rule: None,
            prediction: string_cat
                .get(0)
                .unwrap()
                .to_owned(),
            confidence: probability
                .get(0)
                .unwrap()
                .to_owned()
        }
    );
}

//evaluate the metric on all splits
pub fn evaluate_metric(data: &DataFrame, feature: &str, target: &str) -> PolarsResult<DataFrame> {
    // grabs the unique values
    let values = data.column(feature)?;
    let unique = values.unique()?;

    // create a lagged column to identify split points
    let split = df!(feature => unique)?
        .lazy()
        .with_columns([((col(feature) + col(feature).shift(lit(-1))) / lit(2.0)).alias("split")])
        .collect()?;
    let split_values: Vec<f64> = split
        .column("split")?
        .f64()?
        .iter()
        .flatten() // drop missing values created by lag
        .collect();

    // iterate over split points
    let metrics: PolarsResult<Series> = split_values
        .iter()
        .map(|sp| {
            // split dataframe
            let higher = data.clone().filter(&values.gt_eq(*sp)?)?;
            let lower = data.clone().filter(&values.lt(*sp)?)?;

            // calculate metrics
            let higher_metric = estimate_gini(&higher, target)?;
            let lower_metric = estimate_gini(&lower, target)?;

            Ok(((higher.shape().0 as f64) * higher_metric
                + (lower.shape().0 as f64) * lower_metric)
                / (values.len() as f64))
        })
        .collect();

    // return a dataframe with a metric evaluation
    // for each split point
    return Ok(df!(
        "split" => Series::new("split", split_values),
        "metrics" => metrics?,
    )?);
}

pub fn evaluate_best_split(
    data: &DataFrame,
    features: Vec<&str>,
    target: &str,
) -> PolarsResult<Rule> {
    // iteratively evaluate the metric on all features
    let metrics: PolarsResult<Vec<LazyFrame>> = features
        .iter()
        .map(|feature| {
            Ok(evaluate_metric(&data, feature, target)?
                .lazy()
                .with_column(feature.lit().alias("feature")))
        })
        .collect();

    // join all results in a single dataframe
    let concat_rules = UnionArgs {
        parallel: true,
        rechunk: true,
        to_supertypes: true,
    };
    let concat_metrics: DataFrame = concat(metrics?, concat_rules)?.collect()?;
    println!(
        "\nconcat_metrics\n{1:->0$}{2:?}{1:-<0$}\n",
        20, "\n", concat_metrics
    );

    // search for the best split
    let expr: Expr = col("metrics").lt_eq(col("metrics").min());
    let best_split: DataFrame = concat_metrics
        .clone()
        .lazy()
        .filter(expr)
        .select([col("feature"), col("split"), col("metrics")])
        .collect()?;
    println!(
        "\nbest_split\n{1:->0$}{2:?}{1:-<0$}\n",
        20, "\n", best_split
    );

    let chosen_features: Vec<String> = best_split
        .column("feature")?
        .categorical()?
        .iter_str()
        .flatten()
        .map(|name| <&str as Into<String>>::into(name))
        .collect();

    let chosen_split_point: f64 = best_split.column("split")?.f64()?.get(0).unwrap();

    let split_metric: f64 = best_split.column("metric")?.f64()?.get(0).unwrap();
    Ok(Rule {
        dimension: chosen_features.get(0).unwrap().to_string(),
        cutoff: chosen_split_point,
        metric: split_metric,
    })
}

#[cfg(test)]
mod test {
}