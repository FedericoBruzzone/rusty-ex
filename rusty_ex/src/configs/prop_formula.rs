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
    // Used to indicate an invalid formula or to invalidate a formula.
    None,
}

impl<T: Clone + Debug> Default for PropFormula<T> {
    fn default() -> Self {
        PropFormula::None
    }
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
            None => panic!("Invalid formula."),
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
            None => panic!("Invalid formula."),
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
                p.push_negation_inwards(); // FIXME: Check correctness
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
            None => panic!("Invalid formula."),
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
    pub fn distribute_disjunction_over_conjunction(&mut self) {
        use PropFormula::*;
        match self {
            Var(_) => {}
            Not(v) => assert!(matches!(**v, Var(_))),
            And(p, q) => {
                p.distribute_disjunction_over_conjunction();
                q.distribute_disjunction_over_conjunction();
            }
            Or(p, q) => {
                match (&mut **p, &mut **q) {
                    (And(p1, q1), _) => {
                        let p1 = bx!(Or(p1.clone(), q.clone()));
                        let q1 = bx!(Or(q1.clone(), q.clone()));
                        *self = And(p1, q1);
                        self.distribute_disjunction_over_conjunction();
                    }
                    (_, And(p2, q2)) => {
                        let p2 = bx!(Or(p.clone(), p2.clone()));
                        let q2 = bx!(Or(p.clone(), q2.clone()));
                        *self = And(p2, q2);
                        self.distribute_disjunction_over_conjunction();
                    }
                    _ => {
                        p.distribute_disjunction_over_conjunction();
                        q.distribute_disjunction_over_conjunction();
                    }
                }
            }
            None => panic!("Invalid formula."),
            _ => unreachable!("The `distribute_disjunction_over_conjunction` function should call only after the `eliminate_iff`, `eliminate_implies`, and `push_negation_inwards` functions."),
        }
    }

    /// Convert the propositional formula to CNF.
    ///
    /// NOTE: `distribute_disjunction_over_conjunction`, `push_negation_inwards`,
    /// `eliminate_implies`, and `eliminate_iff` functions should be called first.
    pub fn to_cnf(&mut self) {
        self.eliminate_iff();
        self.eliminate_implies();
        self.push_negation_inwards();
        self.distribute_disjunction_over_conjunction();
    }

    /// Convert the propositional formula to CNF representation.
    ///
    /// It calls the `to_cnf` function first. So, it is safe to call this function directly.
    ///
    /// This ivalidates the formula.
    pub fn to_cnf_repr(&mut self) -> Cnf<T> {
        self.to_cnf();

        use PropFormula::*;
        // Invalidates the formula.
        match std::mem::take(self) {
            Var(var) => vec![vec![(var.clone(), true)]],
            Not(var) => {
                assert!(matches!(*var, Var(_)));
                if let PropFormula::Var(v) = (*var).clone() {
                    vec![vec![(v.clone(), false)]]
                } else {
                    unreachable!()
                }
            }
            And(mut p, mut q) => {
                let mut cnf = p.to_cnf_repr();
                cnf.extend(q.to_cnf_repr());
                cnf
            }
            Or(mut p, mut q) => {
                let mut cnf = vec![];
                let p_cnf = p.to_cnf_repr();
                let q_cnf = q.to_cnf_repr();
                for p_clause in p_cnf {
                    for q_clause in q_cnf.iter() {
                        let mut clause = p_clause.clone();
                        clause.extend_from_slice(q_clause);
                        cnf.push(clause);
                    }
                }
                cnf
            }
            None => panic!("Invalid formula."),
            _ => unreachable!(),
        }
    }
}
