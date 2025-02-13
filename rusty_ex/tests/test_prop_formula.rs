#![feature(rustc_private)]

mod utils;

use pretty_assertions::assert_eq;
use rusty_ex::configs::prop_formula::PropFormula;
use rusty_ex::configs::CnfFormula;
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
        And(vec![
            Implies(bx!(Var(0)), bx!(Var(1))),
            Implies(bx!(Var(1)), bx!(Var(0)))
        ])
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
        And(vec![
            Implies(
                bx!(And(vec![
                    Implies(bx!(Var(0)), bx!(Var(1))),
                    Implies(bx!(Var(1)), bx!(Var(0)))
                ])),
                bx!(Var(2))
            ),
            Implies(
                bx!(Var(2)),
                bx!(And(vec![
                    Implies(bx!(Var(0)), bx!(Var(1))),
                    Implies(bx!(Var(1)), bx!(Var(0)))
                ]))
            )
        ])
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

    assert_eq!(prop_formula, Or(vec![Not(bx!(Var(0))), Var(1)]));

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
        Or(vec![Not(bx!(Or(vec![Not(bx!(Var(0))), Var(1)]))), Var(2)])
    );

    Ok(())
}

#[test]
fn test_push_negation_inwards_not() -> Result<(), String> {
    use PropFormula::*;

    // !!P
    let mut prop_formula = Not(bx!(Not(bx!(Var(0)))));
    // P
    prop_formula.push_negation_inwards();

    assert_eq!(prop_formula, Var(0));

    Ok(())
}

#[test]
fn test_push_negation_inwards_and() -> Result<(), String> {
    use PropFormula::*;

    // !(P & Q)
    let mut prop_formula = Not(bx!(And(vec![Var(0), Var(1)])));
    // !P | !Q
    prop_formula.push_negation_inwards();

    assert_eq!(prop_formula, Or(vec![Not(bx!(Var(0))), Not(bx!(Var(1)))]));

    Ok(())
}

#[test]
fn test_push_negation_inwards_or() -> Result<(), String> {
    use PropFormula::*;

    // !(P | Q)
    let mut prop_formula = Not(bx!(Or(vec![Var(0), Var(1)])));
    // !P & !Q
    prop_formula.push_negation_inwards();

    assert_eq!(prop_formula, And(vec![Not(bx!(Var(0))), Not(bx!(Var(1)))]));

    Ok(())
}

#[test]
fn test_push_negation_inwards_not_add() -> Result<(), String> {
    use PropFormula::*;

    // !!!P
    let mut prop_formula = Not(bx!(Not(bx!(Not(bx!(Var(0)))))));
    // !P
    prop_formula.push_negation_inwards();

    assert_eq!(prop_formula, Not(bx!(Var(0))));

    Ok(())
}

#[test]
fn test_push_negation_inwards_and_add() -> Result<(), String> {
    use PropFormula::*;

    // !((P | Q) & R)
    let mut prop_formula = Not(bx!(And(vec![Or(vec![Var(0), Var(1)]), Var(2)])));
    // (!P & !Q) | !R
    prop_formula.push_negation_inwards();

    assert_eq!(
        prop_formula,
        Or(vec![
            And(vec![Not(bx!(Var(0))), Not(bx!(Var(1)))]),
            Not(bx!(Var(2)))
        ])
    );

    Ok(())
}

#[test]
fn test_push_negation_inwards_or_add() -> Result<(), String> {
    use PropFormula::*;

    // !((P & Q) | R)
    let mut prop_formula = Not(bx!(Or(vec![And(vec![Var(0), Var(1)]), Var(2)])));
    // (!P | !Q) & !R
    prop_formula.push_negation_inwards();

    assert_eq!(
        prop_formula,
        And(vec![
            Or(vec![Not(bx!(Var(0))), Not(bx!(Var(1)))]),
            Not(bx!(Var(2)))
        ])
    );

    Ok(())
}

#[test]
fn test_distribute_over_conjunction_1() -> Result<(), String> {
    use PropFormula::*;

    // P | (Q & R)
    let mut prop_formula = Or(vec![Var(0), And(vec![Var(1), Var(2)])]);

    // (P | Q) & (P | R)
    prop_formula.distribute_disjunction_over_conjunction();

    assert_eq!(
        prop_formula,
        And(vec![Or(vec![Var(0), Var(1)]), Or(vec![Var(0), Var(2)])])
    );

    Ok(())
}

#[test]
fn test_distribute_over_conjunction_2() -> Result<(), String> {
    use PropFormula::*;

    // (P & Q) | R
    let mut prop_formula = Or(vec![And(vec![Var(0), Var(1)]), Var(2)]);

    // (P | R) & (Q | R)
    prop_formula.distribute_disjunction_over_conjunction();

    assert_eq!(
        prop_formula,
        And(vec![Or(vec![Var(0), Var(2)]), Or(vec![Var(1), Var(2)])])
    );

    Ok(())
}

#[test]
fn test_distribute_over_conjunction_add1() -> Result<(), String> {
    use PropFormula::*;

    // (P & Q) | (R & S)
    let mut prop_formula = Or(vec![And(vec![Var(0), Var(1)]), And(vec![Var(2), Var(3)])]);

    // (P | (R & S)) & (Q | (R & S))
    // (P | R) & (P | S) & (Q | R) & (Q | S)
    prop_formula.distribute_disjunction_over_conjunction();

    assert_eq!(
        prop_formula,
        And(vec![
            Or(vec![Var(0), Var(2)]),
            Or(vec![Var(1), Var(2)]),
            Or(vec![Var(0), Var(3)]),
            Or(vec![Var(1), Var(3)])
        ])
    );

    Ok(())
}

#[test]
fn test_distribute_over_conjunction_add2() -> Result<(), String> {
    use PropFormula::*;

    // P | ((P & Q) | (Q & R))
    let mut prop_formula = Or(vec![
        Var(0),
        Or(vec![And(vec![Var(0), Var(1)]), And(vec![Var(1), Var(2)])]),
    ]);

    prop_formula.distribute_disjunction_over_conjunction();

    assert_eq!(
        prop_formula,
        And(vec![
            Or(vec![Var(0), Or(vec![Var(0), Var(1)])]),
            Or(vec![Var(0), Or(vec![Var(1), Var(1)])]),
            Or(vec![Var(0), Or(vec![Var(0), Var(2)])]),
            Or(vec![Var(0), Or(vec![Var(1), Var(2)])]),
        ])
    );
    Ok(())
}

#[test]
fn test_to_cnf() -> Result<(), String> {
    use PropFormula::*;

    // P | (Q & R)
    let mut prop_formula = Or(vec![Var(0), And(vec![Var(1), Var(2)])]);

    // (P | Q) & (P | R)
    prop_formula.to_cnf();

    assert_eq!(
        prop_formula,
        And(vec![Or(vec![Var(0), Var(1)]), Or(vec![Var(0), Var(2)])])
    );

    Ok(())
}

#[test]
fn test_to_cnf_2() -> Result<(), String> {
    use PropFormula::*;

    // P <-> Q
    let mut prop_formula = Iff(bx!(Var(0)), bx!(Var(1)));

    // (!P | Q) & (!Q | P)
    prop_formula.to_cnf();

    assert_eq!(
        prop_formula,
        And(vec![
            Or(vec![Not(bx!(Var(0))), Var(1)]),
            Or(vec![Not(bx!(Var(1))), Var(0)])
        ])
    );

    Ok(())
}

#[test]
fn test_to_cnf_repr() -> Result<(), String> {
    use PropFormula::*;

    // P | (Q & R)
    let mut prop_formula = Or(vec![Var(0), And(vec![Var(1), Var(2)])]);

    // (P | Q) & (P | R)
    let (cnf, _): (CnfFormula<u32>, _) = prop_formula.to_cnf_repr(false);

    assert_eq!(cnf, [[(0, true), (1, true)], [(0, true), (2, true)]]);

    Ok(())
}

#[test]
fn test_to_cnf_repr_2() -> Result<(), String> {
    use PropFormula::*;

    // P <-> Q
    let mut prop_formula = Iff(bx!(Var(0)), bx!(Var(1)));

    // (!P | Q) & (!Q | P)
    let (cnf, _): (CnfFormula<u32>, _) = prop_formula.to_cnf_repr(false);

    assert_eq!(cnf, [[(0, false), (1, true)], [(1, false), (0, true)]]);

    Ok(())
}
