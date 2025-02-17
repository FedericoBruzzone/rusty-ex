#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::centrality::CentralityMeasures;
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
        for (expected, computed) in $expected.iter().zip($computed.iter()) {
            assert_almost_equal!(expected, computed, $tolerance);
        }
    };
}

macro_rules! assert_almost_equal_option_iter {
    ($expected:expr, $computed:expr, $tolerance:expr) => {
        for (expected, computed) in $expected.iter().zip($computed.iter()) {
            assert_almost_equal!(expected.unwrap(), computed.unwrap(), $tolerance);
        }
    };
}

fn get_centrality_measures(file: &str) -> Result<CentralityMeasures, String> {
    let snippet = &std::fs::read_to_string(format!("{CENTRALITY_FOLDER}/{file}")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--serialized-centrality", "all"])?;
    let deserialized_centrality: CentralityMeasures = serde_json::from_str(&output).unwrap();
    Ok(deserialized_centrality)
}

#[test]
fn test_no_centrality() -> Result<(), String> {
    let measures = get_centrality_measures("no_centrality.rs")?;
    let katz = measures.katz.unwrap();
    let closeness = measures.closeness;
    let eigenvector = measures.eigenvector.unwrap();

    assert_eq!(katz, vec![0.0, 0.0, 0.0]);
    assert_eq!(closeness, vec![Some(0.0), Some(0.0), Some(0.0)]);
    assert_eq!(eigenvector, vec![0.0, 0.0, 0.0]);

    Ok(())
}

#[test]
fn test_nested_features() -> Result<(), String> {
    let measures = get_centrality_measures("nested_features.rs")?;

    let katz = measures.katz.unwrap();
    let closeness = measures.closeness;
    let eigenvector = measures.eigenvector.unwrap();

    let katz_out = vec![0.4416, 0.0, 0.0, 0.0746, 0.0507];
    let closeness_out = vec![
        Some(0.625),
        Some(0.0),
        Some(0.0),
        Some(0.0961),
        Some(0.0591),
    ];
    let eigenvector_out = vec![0.6203, 0.0, 0.0, 0.0788, 0.0162];

    assert_almost_equal_iter!(katz_out, katz, 1e-4);
    assert_almost_equal_option_iter!(closeness_out, closeness, 1e-4);
    assert_almost_equal_iter!(eigenvector_out, eigenvector, 1e-4);

    Ok(())
}
