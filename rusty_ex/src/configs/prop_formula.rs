use std::fmt::Debug;

use crate::utils::bx;

use super::Cnf;

/// Trait for converting a type to a propositional formula.
pub trait ToPropFormula<T> {
    fn to_prop_formula(&self) -> PropFormula<T>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropFormula<T> {
    Var(T),
    Not(Box<PropFormula<T>>),
    And(Box<PropFormula<T>>, Box<PropFormula<T>>),
    Or(Box<PropFormula<T>>, Box<PropFormula<T>>),
    Implies(Box<PropFormula<T>>, Box<PropFormula<T>>),
    Iff(Box<PropFormula<T>>, Box<PropFormula<T>>),
}

impl<T: Clone + Debug> PropFormula<T> {
    /// Eliminate the biconditional operator.
    ///
    /// For instance:
    /// P <-> Q is equivalent to (P -> Q) & (Q -> P)
    pub fn eliminate_iff(&mut self) {
        use PropFormula::*;
        match self {
            Var(_) => {}
            Not(p) => p.eliminate_iff(),
            And(p, q) => {
                p.eliminate_iff();
                q.eliminate_iff();
            }
            Or(p, q) => {
                p.eliminate_iff();
                q.eliminate_iff();
            }
            Implies(p, q) => {
                p.eliminate_iff();
                q.eliminate_iff();
            }
            Iff(p, q) => {
                p.eliminate_iff();
                q.eliminate_iff();
                let new_p = bx!(Implies(p.clone(), q.clone()));
                let new_q = bx!(Implies(q.clone(), p.clone()));
                *self = And(new_p, new_q);
            }
        }
    }

    /// Eliminate the implication operator.
    ///
    /// NOTE: `eliminate_iff` function should be called first.
    ///
    /// For instance:
    /// P -> Q is equivalent to !P | Q
    pub fn eliminate_implies(&mut self) {
        use PropFormula::*;
        match self {
            Var(_) => {}
            Not(p) => p.eliminate_implies(),
            And(p, q) => {
                p.eliminate_implies();
                q.eliminate_implies();
            }
            Or(p, q) => {
                p.eliminate_implies();
                q.eliminate_implies();
            }
            Implies(p, q) => {
                p.eliminate_implies();
                q.eliminate_implies();
                let p = bx!(Not(p.clone()));
                *self = Or(p, q.clone());
            }
            Iff(p, q) => {
                p.eliminate_implies();
                q.eliminate_implies();
            }
        }
    }

    /// Push negation inwards (De Morgan's laws).
    /// It operate recursively on the formula until it reaches a fixed point.
    ///
    /// NOTE: `eliminate_implies` and `eliminate_iff` functions should be called first.
    ///
    /// For instance:
    /// !(P & Q) is equivalent to !P | !Q
    /// !(P | Q) is equivalent to !P & !Q
    /// !!P is equivalent to P
    pub fn push_negation_inwards(&mut self) {
        use PropFormula::*;
        match self {
            Var(_) => {}
            Not(p) => {
                p.push_negation_inwards();
                match **p {
                    Var(_) => {}
                    Not(ref p) => {
                        *self = *p.clone();
                    }
                    And(ref p, ref q) => {
                        let p = bx!(Not(p.clone()));
                        let q = bx!(Not(q.clone()));
                        *self = Or(p, q);
                        self.push_negation_inwards();
                    }
                    Or(ref p, ref q) => {
                        let p = bx!(Not(p.clone()));
                        let q = bx!(Not(q.clone()));
                        *self = And(p, q);
                        self.push_negation_inwards();
                    }
                    _ => unreachable!("The `push_negation_inwards` function should call only after the `eliminate_iff` and `eliminate_implies` functions."),
                }
            }
            And(p, q) => {
                p.push_negation_inwards();
                q.push_negation_inwards();
            }
            Or(p, q) => {
                p.push_negation_inwards();
                q.push_negation_inwards();
            }
            _ => unreachable!("The `push_negation_inwards` function should call only after the `eliminate_iff` and `eliminate_implies` functions."),
        }
    }

    /// Distribute disjunction over conjunction.
    ///
    /// NOTE: `push_negation_inwards`, `eliminate_implies`, and `eliminate_iff` functions should be
    /// called first.
    ///
    /// For instance:
    /// P | (Q & R) is equivalent to (P | Q) & (P | R)
    /// (P & Q) | R is equivalent to (P | R) & (Q | R)
    fn distribute_disjunction_over_conjunction(&mut self) {
        use PropFormula::*;
        match self {
            Var(_) => {}
            And(p, q) => {
                p.distribute_disjunction_over_conjunction();
                q.distribute_disjunction_over_conjunction();
            }
            Or(p, q) => {
                p.distribute_disjunction_over_conjunction();
                q.distribute_disjunction_over_conjunction();
                match (&mut **p, &mut **q) {
                    (And(p1, q1), And(p2, q2)) => {
                        let p1 = bx!(Or(p1.clone(), p2.clone()));
                        let q1 = bx!(Or(q1.clone(), q2.clone()));
                        *self = And(p1, q1);
                    }
                    (And(p1, q1), _) => {
                        let p1 = bx!(Or(p1.clone(), q.clone()));
                        let q1 = bx!(Or(q1.clone(), q.clone()));
                        *self = And(p1, q1);
                    }
                    (_, And(p2, q2)) => {
                        let p2 = bx!(Or(p.clone(), p2.clone()));
                        let q2 = bx!(Or(p.clone(), q2.clone()));
                        *self = And(p2, q2);
                    }
                    _ => {}
                }
            }
            _ => unreachable!("The `distribute_disjunction_over_conjunction` function should call only after the `eliminate_iff`, `eliminate_implies`, and `push_negation_inwards` functions."),
        }
    }

    /// Convert the propositional formula to CNF.
    ///
    /// NOTE: `distribute_disjunction_over_conjunction`, `push_negation_inwards`,
    /// `eliminate_implies`, and `eliminate_iff` functions should be called first.
    ///
    /// This is an inner function for the public `to_cnf` function.
    fn to_cnf_inner(&self) -> Cnf<T> {
        use PropFormula::*;
        match self {
            Var(var) => vec![vec![(var.clone(), true)]],
            Not(var) => {
                assert!(matches!(**var, Var(_)));
                todo!()
            }
            _ => todo!(),
        }
    }

    /// Convert the propositional formula to CNF.
    pub fn to_cnf(&mut self) -> Cnf<T> {
        self.to_cnf_inner()
    }
}
