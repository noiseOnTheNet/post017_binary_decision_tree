mod btree;
use polars::lazy::dsl::Expr;
use polars::prelude::*;
use polars::series::Series;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::ops::Deref;

#[derive(Debug)]
pub struct Rule {
    dimension: String,
    cutoff: f64,
    metric: f64,
}

// categorical types are mapped to u32 because:
// 1. do not are equivalent to rust enums which are actually sum types
// 2. we target also 64bit execution platforms like webasm
#[derive(Debug)]
struct Decision {
    rule: Option<Rule>,
    confidence: f64,
    prediction: u32,
}

struct DTreeBuilder {
    max_level: usize,
    min_size: usize,
}

// uses a struct to define trees constraints
impl DTreeBuilder {
    fn build_node(
        &self,
        data: DataFrame,
        level: usize,
        features: HashSet<&str>,
        target: &str,
    ) -> Option<btree::Node<Decision>> {
        let selection: &UInt32Chunked = data.column(target).unwrap().u32().unwrap();
        let node = predict_majority(&mut selection.iter())?;
        Some(btree::Node::new(node))
    }
    pub fn build<'a>(
        &self,
        data: DataFrame,
        features: HashSet<&str>,
        target: &str,
    ) -> btree::Tree<Decision> {
        if let Some(node) = self.build_node(data, 1, features, target) {
            btree::Tree::from_node(node)
        } else {
            btree::Tree::new()
        }
    }
}

// when creating a node first check which would be the prodicted outcome
fn predict_majority<'a>(values: &mut dyn Iterator<Item = Option<u32>>) -> Option<Decision> {
    let summary: HashMap<u32, u32> = count_groups(values);
    let (prediction, count, total) =
        summary
            .iter()
            .fold((None, 0, 0), |(result, count, total), (key, value)| {
                if *value > count {
                    (Some(key), *value, total + value)
                } else {
                    (result, count, total + value)
                }
            });
    if let Some(result) = prediction {
        Some(Decision {
            rule: None,
            confidence: count as f64 / total as f64,
            prediction: *result,
        })
    } else {
        None
    }
}

pub fn count_groups(values: &mut dyn Iterator<Item = Option<u32>>) -> HashMap<u32, u32> {
    values
        .filter_map(|s| s)
        .fold(HashMap::new(), |mut result, value| {
            result.insert(value, result.get(&value).unwrap_or(&0) + 1);
            result
        })
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
pub fn predict_majority_dataframe<'a>(data: &'a DataFrame, target: &str) -> PolarsResult<String> {
    // extract the categorical target column
    let labels = data.column(target)?.categorical()?;

    // count all categories and sort them
    let result_count = labels.value_counts()?;
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", result_count);

    // get the most frequent category
    let result_cat = result_count.column(target)?.head(Some(1));
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", result_cat);

    // transform the series into a categorical vector
    let actual_cat = result_cat.categorical()?;

    // collect all categories as strings
    let string_cat: Vec<String> = actual_cat
        .iter_str()
        .flatten()
        .map(|name| (*name).into())
        .collect();
    println!("{1:->0$}{2:?}{1:-<0$}", 20, "\n", string_cat);

    // return the most common category as a string
    return Ok(string_cat.get(0).unwrap().deref().into());
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
    use crate::{count_groups, predict_majority};

    #[test]
    fn test_count_groups() {
        let input: [Option<u32>; 14] = [
            Some(1u32),
            Some(1u32),
            Some(3u32),
            Some(2u32),
            None,
            Some(1u32),
            None,
            Some(2u32),
            Some(3u32),
            None,
            Some(2u32),
            Some(2u32),
            None,
            None,
        ];
        let result = count_groups(&mut input.iter().map(|s| *s));
        println!("{:?}", result);
        assert_eq!(result.get(&2u32), Some(&4u32));
    }

    #[test]
    fn test_predict_majority() {
        let input: [Option<u32>; 14] = [
            Some(1u32),
            Some(1u32),
            Some(3u32),
            Some(2u32),
            None,
            Some(1u32),
            None,
            Some(2u32),
            Some(3u32),
            None,
            Some(2u32),
            Some(2u32),
            None,
            None,
        ];
        let result = predict_majority(&mut input.iter().map(|s| *s));
        assert!(result.is_some());
        let result = result.unwrap();
        assert_eq!(result.prediction, 2u32);
        assert!(result.confidence < 0.5);
        assert!(result.confidence > 0.4);
    }
}
