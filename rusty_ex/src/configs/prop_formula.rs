use std::collections::HashMap;
use std::hash::Hash;
use std::{fmt::Debug, ops::Add};

use crate::utils::bx;

use super::CnfFormula;

/// The method to be used for the conversion.
/// It is meant to be used user-side.
pub enum ConversionMethod {
    Naive,
}

/// Trait for converting a type to a propositional formula.
pub trait ToPropFormula<T> {
    fn to_prop_formula(&self, method: ConversionMethod) -> PropFormula<T>;
}

/// Trait to represent an ordinal type, it aims to extend `Enumerable` to infinite sets.
///
/// An ordinal type is a type that can be incremented and decremented.
/// The identity of the type is the `Default` trait.
///
/// Note that, not all `Ordinals` are `Countable`, because of the "first uncountable ordinal" (w1)
/// which contains all countable ordinals.
/// But all `Countables` are `Ordinals`.
pub trait Ordinal: Default + Ord + Add<Output = Self> {
    fn suc(&mut self);
    fn pred(&mut self);
}

impl Ordinal for u32 {
    fn suc(&mut self) {
        *self += 1;
    }

    fn pred(&mut self) {
        *self -= 1;
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PropFormula<T> {
    Var(T),
    Not(Box<PropFormula<T>>),
    And(Vec<PropFormula<T>>),
    Or(Vec<PropFormula<T>>),
    Implies(Box<PropFormula<T>>, Box<PropFormula<T>>),
    Iff(Box<PropFormula<T>>, Box<PropFormula<T>>),
    // Used to indicate an invalid formula or to invalidate a formula.
    #[default]
    None,
}

impl<T: Clone + Debug + Eq + Hash> PropFormula<T> {
    /// Eliminate the biconditional operator.
    ///
    /// For instance:
    /// P <-> Q is equivalent to (P -> Q) & (Q -> P)
    pub fn eliminate_iff(&mut self) {
        fn inner<T: Clone>(f: &mut PropFormula<T>) {
            use PropFormula::*;
            match f {
                Var(_) => {}
                Not(p) => inner(p),
                And(v) => {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                Or(v) => {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                Implies(p, q) => {
                    inner(p);
                    inner(q);
                }
                Iff(p, q) => {
                    inner(p);
                    inner(q);
                    let new_p = Implies(p.clone(), q.clone());
                    let new_q = Implies(q.clone(), p.clone());
                    *f = And(vec![new_p, new_q]);
                }
                None => panic!("Invalid formula."),
            }
        }

        loop {
            let prev = self.clone();
            inner(self);
            if prev == *self {
                break;
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
        fn inner<T: Clone>(f: &mut PropFormula<T>) {
            use PropFormula::*;
            match f {
                Var(_) => {}
                Not(p) => inner(p),
                And(v) => {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                Or(v) => {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                Implies(p, q) => {
                    inner(p);
                    inner(q);
                    let not_p = Not((*p).clone());
                    *f = Or(vec![not_p, *q.clone()]);
                }
                Iff(p, q) => {
                    inner(p);
                    inner(q);
                }
                None => panic!("Invalid formula."),
            }
        }

        loop {
            let prev = self.clone();
            inner(self);
            if prev == *self {
                break;
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
        fn inner<T: Clone>(f: &mut PropFormula<T>) {
            use PropFormula::*;
            match f {
                Var(_) => {}
                Not(p) => {
                    match (**p).clone() {
                        Var(_) => {}
                        Not(ref p) => {
                            *f = *p.clone();
                        }
                        And(v) => {
                            let mut not_v = Vec::new();
                            for f in v.iter() {
                                not_v.push(Not(bx!(f.clone())));
                            }
                            *f = Or(not_v);
                        }
                        Or(v) => {
                            let mut not_v = Vec::new();
                            for f in v.iter() {
                                not_v.push(Not(bx!(f.clone())));
                            }
                            *f = And(not_v);
                        }
                        _ => unreachable!("The `push_negation_inwards` function should call only after the `eliminate_iff` and `eliminate_implies` functions."),
                    }
                }
                And(v) => {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                Or(v) =>  {
                    for f in v.iter_mut() {
                        inner(f);
                    }
                }
                None => panic!("Invalid formula."),
                _ => unreachable!("The `push_negation_inwards` function should call only after the `eliminate_iff` and `eliminate_implies` functions."),
            }
        }

        loop {
            let prev = self.clone();
            inner(self);
            if prev == *self {
                break;
            }
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
        fn inner<T: Clone>(f: &mut PropFormula<T>) {
            use PropFormula::*;
            match f {
                Var(_) => {}
            Not(v) => assert!(matches!(**v, Var(_))),
            And(v) => {
                for f in v.iter_mut() {
                    inner(f);
                }
            }
            Or(v) => {
                // Check if there is a conjunction inside the disjunction.
                if v.iter().any(|f| matches!(f, And(_))) {
                    let mut new_v = Vec::new();
                    // (a & b & c) | (d & e & f) => (a | d) & (a | e) & (a | f) & (b | d) & (b | e)
                    // & (b | f) & (c | d) & (c | e) & (c | f)
                    for f in v.iter_mut() {
                        match f {
                            And(v) => {
                                new_v.push(v.clone())
                            },
                            Var(_) => new_v.push(vec![f.clone()]),
                                Not(v) => {
                                    assert!(matches!(**v, Var(_)));
                                    new_v.push(vec![f.clone()]);
                                }
                                _ => unreachable!(),
                            }
                        }

                        fn cartesian_product<T: Clone>(v: Vec<Vec<T>>) -> Vec<Vec<T>> {
                            let mut result = vec![vec![]];
                            for i in v {
                                let mut new_result = vec![];
                                for j in i {
                                    for k in result.iter() {
                                        let mut new_k = k.clone();
                                        new_k.push(j.clone());
                                        new_result.push(new_k);
                                    }
                                }
                                result = new_result;
                            }
                            result
                        }

                        let new_or = cartesian_product(new_v);

                        *f = And(new_or.into_iter().map(|v| Or(v)).collect());
                    } else {
                        for f in v.iter_mut() {
                            inner(f);
                        }
                    }
                }
                None => panic!("Invalid formula."),
                _ => unreachable!("The `distribute_disjunction_over_conjunction` function should call only after the `eliminate_iff`, `eliminate_implies`, and `push_negation_inwards` functions."),
            }
        }

        loop {
            let prev = self.clone();
            inner(self);
            if prev == *self {
                break;
            }
        }
    }

    /// Convert the propositional formula to CNF.
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
    /// This invalidates the formula.
    ///
    /// # Arguments
    /// * `U` - The type to be used for the countable type.
    /// * `normalize` - If true, it normalizes the formula. It means that it removes duplicates and sorts the variables.
    ///
    /// # Returns
    /// * A CNF formula.
    /// * A mapping from variables to a countable type.
    pub fn to_cnf_repr<U>(&mut self, normalize: bool) -> (CnfFormula<U>, HashMap<T, U>)
    where
        U: Ordinal + Clone,
    {
        fn get_or_insert_and_increment<T: Clone + Eq + Hash, U: Ordinal + Clone>(
            mapping: &mut std::collections::HashMap<T, U>,
            key: T,
            value: &mut U,
        ) -> U {
            if let Some(v) = mapping.get(&key) {
                v.clone()
            } else {
                let res = value.clone();
                mapping.insert(key, res.clone());
                value.suc();
                res
            }
        }

        // Convert the propositional formula to CNF representation.
        //
        // A likely values for `T` and `U` are `String` and `u32`, respectively.
        // That is, `PropFormula<String>` to `CnfFormula<u32>`.
        //
        // # Arguments
        //
        // * `f` - The propositional formula.
        // * `mapping` - The mapping from variables to a countable type.
        // * `curr_value` - The current value to be used for the countable type.
        fn rec_to_cnf_repr<T, U>(
            f: &PropFormula<T>,
            mapping: &mut HashMap<T, U>,
            curr_value: &mut U,
        ) -> CnfFormula<U>
        where
            T: Clone + Eq + Hash,
            U: Ordinal + Clone,
        {
            use PropFormula::*;
            match f {
                Var(var) => {
                    let value = get_or_insert_and_increment(mapping, (*var).clone(), curr_value);
                    vec![vec![(value, true)]]
                }
                Not(var) => {
                    assert!(matches!(**var, Var(_)));
                    if let PropFormula::Var(v) = (**var).clone() {
                        let value = get_or_insert_and_increment(mapping, v.clone(), curr_value);
                        vec![vec![(value, false)]]
                    } else {
                        unreachable!()
                    }
                }
                And(prop) => {
                    let mut cnf = vec![];
                    for f in prop {
                        let f_cnf = rec_to_cnf_repr(f, mapping, curr_value);
                        cnf.push(f_cnf.into_iter().flatten().collect());
                    }
                    cnf
                }
                Or(prop) => {
                    assert!(prop.iter().all(|f| matches!(f, Var(_) | Not(_))));
                    let mut cnf = vec![];
                    for f in prop {
                        let f_cnf = rec_to_cnf_repr(f, mapping, curr_value);
                        cnf.extend(f_cnf);
                    }
                    cnf
                }
                None => panic!("Invalid formula."),
                _ => unreachable!(),
            }
        }

        self.to_cnf();
        let mut mapping = HashMap::<T, U>::new();
        let cnf = rec_to_cnf_repr(self, &mut mapping, &mut U::default());

        // Remove duplicates
        if normalize {
            let mut cnf = cnf
                .into_iter()
                .map(|mut v| {
                    v.sort();
                    v.dedup();
                    v
                })
                .collect::<Vec<Vec<(U, bool)>>>();
            cnf.sort();
            cnf.dedup();
            (cnf, mapping)
        } else {
            (cnf, mapping)
        }
    }
}
