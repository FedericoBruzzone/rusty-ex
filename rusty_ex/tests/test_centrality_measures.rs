#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::centrality::CentralityMeasures;
use utils::run_with_cargo_bin_and_snippet;

const CENTRALITY_FOLDER: &str = "tests/snippets/centrality";

fn get_centrality_measures(file: &str) -> Result<CentralityMeasures, String> {
    let snippet = &std::fs::read_to_string(format!("{CENTRALITY_FOLDER}/{file}")).unwrap();
    let (output, _) = run_with_cargo_bin_and_snippet(snippet, &["--serialized-centrality", "all"])?;
    let deserialized_centrality: CentralityMeasures = serde_json::from_str(&output).unwrap();
    Ok(deserialized_centrality)
}

#[test]
fn test_one_in_one() -> Result<(), String> {
    let measures = get_centrality_measures("no_centrality.rs")?;
    let katz = measures.katz.unwrap();
    let closeness = measures.closeness;
    let eigenvector = measures.eigenvector.unwrap();

    assert_eq!(katz, vec![0.0, 0.0, 0.0]);
    assert_eq!(closeness, vec![Some(0.0), Some(0.0), Some(0.0)]);
    assert_eq!(eigenvector, vec![0.0, 0.0, 0.0]);

    Ok(())
}
