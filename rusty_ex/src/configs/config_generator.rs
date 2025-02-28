use super::{config_solver::ConfigSolver, CnfFormula};
use rustsat_minisat::core::Minisat;

/// This struct is used to generate the configurations for the given CNF formula.
pub struct ConfigGenerator<S> {
    /// The solver.
    solver: ConfigSolver<S>,
    /// The CNF formula.
    cnf: CnfFormula<u32>,
    /// Centrality indices.
    indices: Vec<u32>,
    /// The amount of configurations that must be generated.
    amount: usize,
}

impl ConfigGenerator<Minisat> {
    pub fn new(cnf: CnfFormula<u32>, indices: &[u32], amount: usize) -> Self {
        let solver = ConfigSolver::default();
        Self {
            solver,
            cnf,
            indices: indices.to_vec(),
            amount,
        }
    }

    /// Generate the configurations.
    pub fn generate(&mut self) -> Vec<CnfFormula<u32>> {
        let mut configs = Vec::new();
        for index in &self.indices {
            if configs.len() >= self.amount {
                break;
            }
            let var = (*index, true);
            self.solver.add_cnf(self.cnf.clone());
            let sol: CnfFormula<u32> = self.solver.all_configs_given_a_var(vec![var]);
            configs.push(sol);
        }
        configs
    }
}
