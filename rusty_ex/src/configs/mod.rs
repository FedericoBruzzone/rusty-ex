pub mod centrality;
pub mod config_generator;
pub mod config_solver;
pub mod prop_formula;

pub type CnfLit<T> = (T, bool);
pub type CnfClause<T> = Vec<CnfLit<T>>;
pub type CnfFormula<T> = Vec<CnfClause<T>>;
