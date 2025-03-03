#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::centrality::Centrality;
use utils::run_with_cargo_bin_and_snippet;

const CENTRALITY_FOLDER: &str = "tests/snippets/centrality";

macro_rules! assert_almost_equal {
    ($x:expr, $y:expr, $d:expr) => {
        if ($x - $y).abs() >= $d {
            panic!("{} != {} within delta of {}", $x, $y, $d);
        }
    };
}

macro_rules! assert_almost_equal_iter {
    ($expected:expr, $computed:expr, $tolerance:expr) => {
        assert_eq!($expected.len(), $computed.len());
        for (expected, computed) in $expected.iter().zip($computed.iter()) {
            assert_almost_equal!(expected, computed, $tolerance);
        }
    };
}

macro_rules! assert_almost_equal_option_iter {
    ($expected:expr, $computed:expr, $tolerance:expr) => {
        assert_eq!($expected.len(), $computed.len());
        for (expected, computed) in $expected.iter().zip($computed.iter()) {
            assert_almost_equal!(expected.unwrap(), computed.unwrap(), $tolerance);
        }
    };
}

macro_rules! assert_greatest_index {
    ($computed:expr, $index:expr) => {
        let max = $computed
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let index = $computed.iter().position(|&x| x == *max).unwrap();
        assert_eq!(index, $index);
    };
}

fn get_centrality_measures(file: &str) -> Result<Centrality<u32>, String> {
    let snippet = &std::fs::read_to_string(format!("{CENTRALITY_FOLDER}/{file}")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--serialized-centrality", "all"])?;
    let deserialized_centrality: Centrality<u32> = serde_json::from_str(&output).unwrap();
    Ok(deserialized_centrality)
}

#[test]
fn test_no_centrality() -> Result<(), String> {
    let centrality = get_centrality_measures("no_centrality.rs")?;
    let katz = centrality.katz().unwrap();
    let closeness = centrality.closeness();
    let eigenvector = centrality.eigenvector().unwrap();

    assert_eq!(*katz, vec![0.0, 0.0]);
    assert_eq!(*closeness, vec![Some(0.0), Some(0.0)]);
    assert_eq!(*eigenvector, vec![0.0, 0.0]);

    Ok(())
}

#[test]
fn test_nested_features() -> Result<(), String> {
    let centrality = get_centrality_measures("nested_features.rs")?;

    let katz = centrality.katz().unwrap();
    let closeness = centrality.closeness();
    let eigenvector = centrality.eigenvector().unwrap();

    let katz_out = vec![0.14722, 0.34022, 0.3236, 0.1078];
    let closeness_out = vec![Some(0.2083), Some(0.4545), Some(0.5555), Some(0.1388)];
    let eigenvector_out = vec![0.2067, 0.2606, 0.3415, 0.1138];

    assert_almost_equal_iter!(*katz_out, katz, 1e-4);
    assert_almost_equal_option_iter!(*closeness_out, closeness, 1e-4);
    assert_almost_equal_iter!(*eigenvector_out, eigenvector, 1e-4);

    Ok(())
}

#[test]
fn test_one_important_feature() -> Result<(), String> {
    let centrality = get_centrality_measures("one_important_feature.rs")?;

    let katz = centrality.katz().unwrap();
    let closeness = centrality.closeness();
    let eigenvector = centrality.eigenvector().unwrap();

    let katz_out = vec![0.4620, 0.3708, 0.5064, 0.1205];
    let closeness_out = vec![Some(0.625), Some(0.4545), Some(0.8333), Some(0.1190)];
    let eigenvector_out = vec![0.6845, 0.3466, 0.5617, 0.0595];

    assert_almost_equal_iter!(*katz_out, katz, 1e-4);
    assert_almost_equal_option_iter!(*closeness_out, closeness, 1e-4);
    assert_almost_equal_iter!(*eigenvector_out, eigenvector, 1e-4);

    assert_greatest_index!(katz, 2);
    assert_greatest_index!(closeness, 2);
    assert_greatest_index!(eigenvector, 0);

    Ok(())
}

#[test]
fn test_one_important_nested_feature() -> Result<(), String> {
    let centrality = get_centrality_measures("one_important_nested_feature.rs")?;

    let katz = centrality.katz().unwrap();
    let closeness = centrality.closeness();
    let eigenvector = centrality.eigenvector().unwrap();

    let katz_out = vec![0.4909, 0.4865];
    let closeness_out = vec![Some(0.6), Some(0.5)];
    let eigenvector_out = vec![0.5357, 0.4358];

    assert_almost_equal_iter!(*katz_out, katz, 1e-4);
    assert_almost_equal_option_iter!(*closeness, closeness_out, 1e-4);
    assert_almost_equal_iter!(*eigenvector_out, eigenvector, 1e-4);

    assert_greatest_index!(katz, 0);
    assert_greatest_index!(closeness, 0);
    assert_greatest_index!(eigenvector, 0);

    Ok(())
}
