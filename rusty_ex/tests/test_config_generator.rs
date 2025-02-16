#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::config_generator::{ConfigGenerator, ConfigGeneratorUtils};

#[test]
fn test_zero_with_zero_true() -> Result<(), String> {
    let cnf = vec![vec![(0, true)]];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let var = (0, true); // The literal that must be true
    let configs = generator.all_configs_given_a_var(vec![var]);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 1);
    assert_eq!(configs_str, "(0)\n");

    Ok(())
}

#[test]
fn test_zero_with_zero_false() -> Result<(), String> {
    let cnf = vec![vec![(0, true)]];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let var = (0, false); // The literal that must be true
    let configs = generator.all_configs_given_a_var(vec![var]);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 0);
    assert_eq!(configs_str, "");

    Ok(())
}

#[test]
fn complex_1() -> Result<(), String> {
    let cnf = vec![
        vec![(0, true), (1, false), (2, true)],
        vec![(0, false), (1, true)],
        vec![(2, true), (3, false)],
    ];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let var = (1, true); // The literal that must be true
    let configs = generator.all_configs_given_a_var(vec![var]);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 5);
    assert_eq!(
        configs_str,
        "(0 & 1 & 2 & 3)\n\
(0 & 1 & 2 & !3)\n\
(!0 & 1 & 2 & !3)\n\
(!0 & 1 & 2 & 3)\n\
(0 & 1 & !2 & !3)\n"
    );

    Ok(())
}

#[test]
fn test_simple_real_case() -> Result<(), String> {
    let cnf = vec![vec![(0, true), (1, true)], vec![(0, true), (1, true)]];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let var = (0, true); // The literal that must be true
    let configs = generator.all_configs_given_a_var(vec![var]);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 2);
    assert_eq!(configs_str, "(0 & 1)\n(0 & !1)\n");

    Ok(())
}

#[test]
fn test_multiple_var1() -> Result<(), String> {
    let cnf = vec![vec![(0, true), (1, true)], vec![(0, true), (1, true)]];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let vars = vec![(0, true), (1, true)]; // The literal that must be true
    let configs = generator.all_configs_given_a_var(vars);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 1);
    assert_eq!(configs_str, "(0 & 1)\n");

    Ok(())
}

#[test]
fn test_multiple_var2() -> Result<(), String> {
    // (x0 | x1 | x2) & (!x0 | x1) & (x2 | !x3)
    let cnf = vec![
        vec![(0, true), (1, false), (2, true)],
        vec![(0, false), (1, true)],
        vec![(2, true), (3, false)],
    ];
    let mut generator = ConfigGenerator::default();
    generator.add_cnf(cnf);
    let vars = vec![(0, true), (1, true)]; // The literal that must be true
    let configs = generator.all_configs_given_a_var(vars);
    let configs_str = ConfigGeneratorUtils::to_string(&configs);

    assert_eq!(configs.len(), 3);
    assert_eq!(
        configs_str,
        "(0 & 1 & 2 & 3)\n\
(0 & 1 & 2 & !3)\n\
(0 & 1 & !2 & !3)\n"
    );

    Ok(())
}

