#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::prop_formula::PropFormula;

#[test]
fn test_eliminate_iff() -> Result<(), String> {
    use PropFormula::*;

    // P <-> Q
    let mut prop_formula = Iff(Box::new(Var(0)), Box::new(Var(1)));
    // (P -> Q) & (Q -> P)
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        And(
            Box::new(Implies(Box::new(Var(0)), Box::new(Var(1)))),
            Box::new(Implies(Box::new(Var(1)), Box::new(Var(0))))
        )
    );

    Ok(())
}

#[test]
fn test_eliminate_iff_nested() -> Result<(), String> {
    use PropFormula::*;

    // (P <-> Q) <-> R
    let mut prop_formula = Iff(
        Box::new(Iff(Box::new(Var(0)), Box::new(Var(1)))),
        Box::new(Var(2)),
    );
    // (((P -> Q) & (Q -> P)) -> R) & (R -> ((P -> Q) & (Q -> P)))
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        And(
            Box::new(Implies(
                Box::new(And(
                    Box::new(Implies(Box::new(Var(0)), Box::new(Var(1)))),
                    Box::new(Implies(Box::new(Var(1)), Box::new(Var(0))))
                )),
                Box::new(Var(2))
            )),
            Box::new(Implies(
                Box::new(Var(2)),
                Box::new(And(
                    Box::new(Implies(Box::new(Var(0)), Box::new(Var(1)))),
                    Box::new(Implies(Box::new(Var(1)), Box::new(Var(0))))
                ))
            ))
        )
    );

    Ok(())
}

#[test]
fn test_eliminate_implies() -> Result<(), String> {
    use PropFormula::*;

    // P -> Q
    let mut prop_formula = Implies(Box::new(Var(0)), Box::new(Var(1)));
    // !P | Q
    prop_formula.eliminate_implies();

    assert_eq!(
        prop_formula,
        Or(Box::new(Not(Box::new(Var(0)))), Box::new(Var(1)))
    );

    Ok(())
}

#[test]
fn test_eliminate_implies_nested() -> Result<(), String> {
    use PropFormula::*;

    // (P -> Q) -> R
    let mut prop_formula = Implies(
        Box::new(Implies(Box::new(Var(0)), Box::new(Var(1)))),
        Box::new(Var(2)),
    );
    // !(!P | Q) | R
    prop_formula.eliminate_implies();

    assert_eq!(
        prop_formula,
        PropFormula::Or(
            Box::new(Not(Box::new(Or(
                Box::new(Not(Box::new(Var(0)))),
                Box::new(Var(1))
            )))),
            Box::new(Var(2))
        )
    );

    Ok(())
}
