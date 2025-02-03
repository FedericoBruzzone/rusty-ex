#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::prop_formula::PropFormula;

#[test]
fn test_eliminate_iff() -> Result<(), String> {
    // P <-> Q
    let mut prop_formula =
        PropFormula::Iff(Box::new(PropFormula::Var(0)), Box::new(PropFormula::Var(1)));
    // (P -> Q) & (Q -> P)
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        PropFormula::And(
            Box::new(PropFormula::Implies(
                Box::new(PropFormula::Var(0)),
                Box::new(PropFormula::Var(1))
            )),
            Box::new(PropFormula::Implies(
                Box::new(PropFormula::Var(1)),
                Box::new(PropFormula::Var(0))
            ))
        )
    );

    Ok(())
}

#[test]
fn test_eliminate_iff_nested() -> Result<(), String> {
    // (P <-> Q) <-> R
    let mut prop_formula = PropFormula::Iff(
        Box::new(PropFormula::Iff(
            Box::new(PropFormula::Var(0)),
            Box::new(PropFormula::Var(1)),
        )),
        Box::new(PropFormula::Var(2)),
    );
    // (((P -> Q) & (Q -> P)) -> R) & (R -> ((P -> Q) & (Q -> P)))
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        PropFormula::And(
            Box::new(PropFormula::Implies(
                Box::new(PropFormula::And(
                    Box::new(PropFormula::Implies(
                        Box::new(PropFormula::Var(0)),
                        Box::new(PropFormula::Var(1))
                    )),
                    Box::new(PropFormula::Implies(
                        Box::new(PropFormula::Var(1)),
                        Box::new(PropFormula::Var(0))
                    ))
                )),
                Box::new(PropFormula::Var(2))
            )),
            Box::new(PropFormula::Implies(
                Box::new(PropFormula::Var(2)),
                Box::new(PropFormula::And(
                    Box::new(PropFormula::Implies(
                        Box::new(PropFormula::Var(0)),
                        Box::new(PropFormula::Var(1))
                    )),
                    Box::new(PropFormula::Implies(
                        Box::new(PropFormula::Var(1)),
                        Box::new(PropFormula::Var(0))
                    ))
                ))
            ))
        )
    );

    Ok(())
}
