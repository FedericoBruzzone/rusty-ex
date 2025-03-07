use rustsat::{
    solvers::{Solve, SolveStats, SolverResult},
    types::{Clause, Lit, TernaryVal},
};
use rustsat_minisat::core::Minisat;

use super::{CnfClause, CnfFormula, CnfLit};

pub struct ConfigSolver<S> {
    solver: S,
}

impl Default for ConfigSolver<Minisat> {
    fn default() -> Self {
        Self::new(Minisat::default())
    }
}

impl<S> ConfigSolver<S>
where
    S: Solve + SolveStats,
{
    fn new(solver: S) -> Self {
        Self { solver }
    }

    /// Add a clause to the solver.
    ///
    /// For instance:
    ///
    /// `vec![(0, true), (1, false), (2, true)]` is the clause `(x0 | !x1 | x2)` in DIMACS format.
    fn add_clause(&mut self, clause: CnfClause<u32>) -> &mut Self {
        let mut c = Clause::new();
        for (var, neg) in clause {
            c.add(Lit::new(var, neg));
        }
        self.solver.add_clause(c).unwrap();
        self
    }

    /// Add a CNF to the solver.
    ///
    /// For instance:
    ///
    /// `vec![ vec![(0, true), (1, false), (2, true)], vec![(0, false), (1, true)]]` is the CNF `((x0 | !x1 | x2) & (!x0 | x1))` in DIMACS format.
    pub fn add_cnf(&mut self, cnf: CnfFormula<u32>) {
        for clause in cnf {
            self.add_clause(clause);
        }
    }
    /// This function finds all the configurations that satisfy a set of variables.
    ///
    /// For instance:
    /// [(0, true)] is the variable `x0` that must be true, the function fill return
    /// all the configurations that satisfy `x0`.
    /// [(0, false), (1, true)] is the variable `x0` that must be false and `x1` that must be true,
    /// the function fill return all the configurations that satisfy `!x0 & x1`.
    pub fn all_configs_given_a_var(&mut self, vars: Vec<CnfLit<u32>>) -> CnfFormula<u32> {
        // Set the variable to the given value.
        for var in vars {
            self.add_clause(vec![var]);
        }

        // Find all the configurations that satisfy the given variable.
        let mut all_configs = Vec::new();

        loop {
            match self.solver.solve().unwrap() {
                SolverResult::Sat => {
                    let sol = self.solver.full_solution().unwrap();
                    let config = sol
                        .iter()
                        .map(|lit| match sol[lit.var()] {
                            TernaryVal::True => (lit.vidx32(), true),
                            TernaryVal::False => (lit.vidx32(), false),
                            TernaryVal::DontCare => {
                                // The formula is satisfied.
                                // For now, we don't handle this case because
                                // we are only interested in the configurations that satisfy the
                                // given variables.
                                panic!("Unexpected DontCare")
                            }
                        })
                        .collect::<Vec<(u32, bool)>>();
                    all_configs.push(config.clone());

                    // Add the negation of the current configuration.
                    let mut neg_clause = Clause::new();
                    for (var, val) in config {
                        neg_clause.add(Lit::new(var, val));
                    }

                    self.solver.add_clause(neg_clause).unwrap();
                }
                SolverResult::Unsat => break, // No more configurations.
                SolverResult::Interrupted => panic!("Unexpected Interrupted"), // TODO: Should we handle this case?
            }
        }

        all_configs
    }
}

/// ZST for ConfigGenerator
///
/// We need this struct to implement the static function for the `ConfigGenerator`.
pub struct ConfigSolverUtils;

impl ConfigSolverUtils {
    pub fn to_string(configs: &CnfFormula<u32>) -> String {
        let mut s = String::new();
        for clause in configs {
            let clause_str: Vec<String> = clause
                .iter()
                .map(|(idx, neg)| format!("{}{}", if *neg { "!" } else { "" }, idx))
                .collect();
            s.push_str(&format!("({})\n", clause_str.join(" & ")));
        }
        s
    }
}
