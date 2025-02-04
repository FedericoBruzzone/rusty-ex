#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::prop_formula::PropFormula;
use utils::bx;

#[test]
fn test_eliminate_iff() -> Result<(), String> {
    use PropFormula::*;

    // P <-> Q
    let mut prop_formula = Iff(bx!(Var(0)), bx!(Var(1)));
    // (P -> Q) & (Q -> P)
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        And(
            bx!(Implies(bx!(Var(0)), bx!(Var(1)))),
            bx!(Implies(bx!(Var(1)), bx!(Var(0))))
        )
    );

    Ok(())
}

#[test]
fn test_eliminate_iff_nested() -> Result<(), String> {
    use PropFormula::*;

    // (P <-> Q) <-> R
    let mut prop_formula = Iff(bx!(Iff(bx!(Var(0)), bx!(Var(1)))), bx!(Var(2)));
    // (((P -> Q) & (Q -> P)) -> R) & (R -> ((P -> Q) & (Q -> P)))
    prop_formula.eliminate_iff();

    assert_eq!(
        prop_formula,
        And(
            bx!(Implies(
                bx!(And(
                    bx!(Implies(bx!(Var(0)), bx!(Var(1)))),
                    bx!(Implies(bx!(Var(1)), bx!(Var(0))))
                )),
                bx!(Var(2))
            )),
            bx!(Implies(
                bx!(Var(2)),
                bx!(And(
                    bx!(Implies(bx!(Var(0)), bx!(Var(1)))),
                    bx!(Implies(bx!(Var(1)), bx!(Var(0))))
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
    let mut prop_formula = Implies(bx!(Var(0)), bx!(Var(1)));
    // !P | Q
    prop_formula.eliminate_implies();

    assert_eq!(prop_formula, Or(bx!(Not(bx!(Var(0)))), bx!(Var(1))));

    Ok(())
}

#[test]
fn test_eliminate_implies_nested() -> Result<(), String> {
    use PropFormula::*;

    // (P -> Q) -> R
    let mut prop_formula = Implies(bx!(Implies(bx!(Var(0)), bx!(Var(1)))), bx!(Var(2)));
    // !(!P | Q) | R
    prop_formula.eliminate_implies();

    assert_eq!(
        prop_formula,
        PropFormula::Or(
            bx!(Not(bx!(Or(bx!(Not(bx!(Var(0)))), bx!(Var(1)))))),
            bx!(Var(2))
        )
    );

    Ok(())
}
