use rustc_ast::NodeId;
use rustworkx_core::petgraph::dot::{Config, Dot};
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

// -------------------- Features --------------------

/// Simple feature
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct Feature {
    pub name: String,
    pub not: bool,
}

/// Complex feature: none, a single feature (not included), all features, or any feature
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ComplexFeature {
    None,
    Feature(Feature),
    All(Vec<ComplexFeature>),
    Any(Vec<ComplexFeature>),
}

// -------------------- Weights --------------------

/// Type of the weight of a node (not the actual weight, only the type)
#[derive(Debug, Clone)]
pub enum NodeWeightKind {
    /// Leaf, a literal or something that does NOT calls anything.
    /// The weight is 1.0
    Leaf(String),
    /// A block of statements, expressions or items. Has no intrinsic weight.
    /// The weight is the sum of the children
    Block(String),
    /// A call to another node, like a function call.
    /// The weight is the weight of the called thing
    Call(String, Option<String>),
    /// Items that have no weight, like `_`.
    /// The weight is 0.0
    NoWeight(String),
}

/// Weight of a node: not yet calculated, a float, or waiting for something to be resolved
#[derive(Debug, Clone)]
pub enum NodeWeight {
    ToBeCalculated,
    Weight(f64),
    Wait(String),
}

// -------------------- Graphs common --------------------

/// Edge between nodes, has a weight
#[derive(Debug, Clone)]
pub struct Edge {
    pub weight: f64,
}

// -------------------- AST Graph --------------------

/// Index of a node in the AST graph
pub type AstIndex = NodeIndex;

/// Key to uniquely identify an AST node (used to get the index in the `nodes` map)
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct AstKey(pub NodeId);

/// AST node, with features (complex, already parsed) and weight (kind and value).
/// The weight of an AST node is the sum of the weights of all its children plus
/// its intrinsic weight (if it has one)
#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_id: AstKey,
    pub ident: Option<String>,
    pub features: ComplexFeature,
    pub weight_kind: NodeWeightKind,
    pub weight: NodeWeight,
}

/// AST graph: the actual graph and a map to get the index of a node from its key
#[derive(Debug, Clone)]
pub struct AstGraph {
    pub graph: DiGraph<AstNode, Edge>,
    pub nodes: HashMap<AstKey, AstIndex>,
}

// -------------------- Features Graph --------------------

/// Index of a feature node in the features graph
pub type FeatureIndex = NodeIndex;

/// Key to uniquely identify a feature (used to get the index in the `nodes` map)
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct FeatureKey(pub Feature);

/// Feature node, with the feature and the weight (if calculated).
/// The weight is calculated "horizontally", considering only the "siblings"
#[derive(Debug, Clone)]
pub struct FeatureNode {
    pub feature: FeatureKey,
    pub weight: Option<f64>,
}

/// Features graph: the actual graph and a map to get the index of a feature node from its key
#[derive(Debug, Clone)]
pub struct FeaturesGraph {
    pub graph: DiGraph<FeatureNode, Edge>,
    pub nodes: HashMap<FeatureKey, FeatureIndex>,
}

// -------------------- Artifacts Graph --------------------

/// Index of an artifact node in the artifacts graph
pub type ArtifactIndex = NodeIndex;

/// Key to uniquely identify an artifact (used to get the index in the `nodes` map)
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct ArtifactKey(pub NodeId);

/// Artifact node, with features (indexes in the features graph) and weight (sum of the weights
/// of all its children (even the not annotated ones) plus its intrinsic weight (if it has one))
#[derive(Debug, Clone)]
pub struct ArtifactNode {
    pub artifact: ArtifactKey,
    pub ident: Option<String>,
    pub features: Vec<FeatureIndex>, // index in features graph
    pub weight: NodeWeight,
}

/// Artifacts graph: the actual graph and a map to get the index of an artifact node from its key
#[derive(Debug, Clone)]
pub struct ArtifactsGraph {
    pub graph: DiGraph<ArtifactNode, Edge>,
    pub nodes: HashMap<ArtifactKey, ArtifactIndex>,
}

// -------------------- Implementations --------------------

// TODO: fare anche l'update per i 3 grafi e fare refactoring in lib.rs

impl AstGraph {
    /// Create a new empty AST graph
    pub fn new() -> Self {
        AstGraph {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create an AST node in the AST graph and add it to the AST nodes hashmap.
    /// Return the index of the created node
    pub fn create_node(
        &mut self,
        node_id: AstKey,
        ident: Option<String>,
        features: ComplexFeature,
        weight_kind: NodeWeightKind,
        weight: NodeWeight,
    ) -> AstIndex {
        if let Some(index) = self.nodes.get(&node_id) {
            return *index;
        }

        let index = self.graph.add_node(AstNode {
            node_id: node_id.clone(),
            ident,
            features,
            weight_kind,
            weight,
        });
        self.nodes.insert(node_id, index);

        index
    }

    /// Print AST graph in DOT format
    pub fn print_dot(&self) {
        let get_node_attr = |_g: &DiGraph<AstNode, Edge>, node: (NodeIndex, &AstNode)| {
            let index = node.0.index();
            let ast_node = node.1;
            format!(
                "label=\"i{}: node{:?} ({}) '{}' #[{}] {}\"",
                index,
                ast_node.node_id,
                ast_node.weight_kind,
                ast_node.ident.clone().unwrap_or(" ".to_string()),
                ast_node.features,
                ast_node.weight,
            )
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_g, _e| String::new(), // do not print edge labels
                &get_node_attr,
            )
        )
    }
}

impl Default for AstGraph {
    fn default() -> Self {
        AstGraph::new()
    }
}

impl FeaturesGraph {
    /// Create a new empty features graph
    pub fn new() -> Self {
        FeaturesGraph {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create a feature node in the features graph and add it to the features nodes hashmap.
    /// Return the index of the created node
    pub fn create_node(&mut self, feature: FeatureKey, weight: Option<f64>) -> FeatureIndex {
        if let Some(index) = self.nodes.get(&feature) {
            return *index;
        }

        let index = self.graph.add_node(FeatureNode {
            feature: feature.clone(),
            weight,
        });
        self.nodes.insert(feature, index);

        index
    }

    /// Print features graph in DOT format
    pub fn print_dot(&self) {
        let get_node_attr = |_g: &DiGraph<FeatureNode, Edge>, node: (NodeIndex, &FeatureNode)| {
            let index = node.0.index();
            let feature = node.1;
            match feature.feature.0.not {
                true => format!("label=\"i{}: !{}\"", index, feature.feature.0.name),
                false => format!("label=\"i{}: {}\"", index, feature.feature.0.name),
            }
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_g, e| format!("label=\"{:.2}\"", e.weight().weight),
                &get_node_attr,
            )
        )
    }
}

impl Default for FeaturesGraph {
    fn default() -> Self {
        FeaturesGraph::new()
    }
}

impl ArtifactsGraph {
    /// Create a new empty artifacts graph
    pub fn new() -> Self {
        ArtifactsGraph {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create an artifact node in the artifacts graph and add it to the artifacts nodes hashmap.
    /// Return the index of the created node
    pub fn create_node(
        &mut self,
        artifact: ArtifactKey,
        ident: Option<String>,
        features: Vec<FeatureIndex>,
        weight: NodeWeight,
    ) -> ArtifactIndex {
        if let Some(index) = self.nodes.get(&artifact) {
            return *index;
        }

        let index = self.graph.add_node(ArtifactNode {
            artifact: artifact.clone(),
            ident,
            features,
            weight,
        });
        self.nodes.insert(artifact, index);

        index
    }

    /// Print artifacts graph in DOT format
    pub fn print_dot(&self) {
        let get_node_attr = |_g: &DiGraph<ArtifactNode, Edge>, node: (NodeIndex, &ArtifactNode)| {
            let index = node.0.index();
            let artifact = node.1;
            format!(
                "label=\"i{} node{} '{}' #[{}] {}\"",
                index,
                artifact.artifact.0,
                artifact.ident.clone().unwrap_or("-".to_string()),
                artifact
                    .features
                    .iter()
                    .map(|f| f.index().to_string())
                    .collect::<Vec<String>>()
                    .join(", "),
                artifact.weight
            )
        };

        println!(
            "{:?}",
            Dot::with_attr_getters(
                &self.graph,
                &[Config::NodeNoLabel, Config::EdgeNoLabel],
                &|_g, _e| String::new(), // do not print edge labels
                &get_node_attr,
            )
        )
    }
}

impl Default for ArtifactsGraph {
    fn default() -> Self {
        ArtifactsGraph::new()
    }
}

impl Display for ComplexFeature {
    /// Complex feature to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplexFeature::None => write!(f, ""),
            ComplexFeature::Feature(Feature { name, not }) => {
                let name = match not {
                    true => "!".to_string() + name,
                    false => name.to_string(),
                };
                write!(f, "{}", name)
            }
            ComplexFeature::All(features) => write!(
                f,
                "all({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            ComplexFeature::Any(features) => write!(
                f,
                "any({})",
                features
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

impl NodeWeightKind {
    /// Parse the kind variant name from the debug string (keep only the Kind name)
    pub fn parse_kind_variant_name(s: String) -> String {
        s.split(['(', '{']).next().unwrap_or("").trim().to_string()
    }
}

impl Display for NodeWeightKind {
    /// NodeWeightKind to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeWeightKind::Leaf(name) => write!(f, "Leaf({})", name),
            NodeWeightKind::Block(name) => write!(f, "Block({})", name),
            NodeWeightKind::Call(name, ident) => write!(
                f,
                "Call({})->{}",
                name,
                ident.clone().unwrap_or("??".to_string())
            ),
            NodeWeightKind::NoWeight(name) => write!(f, "NoWeight({})", name),
        }
    }
}

impl Display for NodeWeight {
    /// ASTNodeWeight to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeWeight::ToBeCalculated => write!(f, "w-"),
            NodeWeight::Weight(w) => write!(f, "w{:.2}", w),
            NodeWeight::Wait(wait_ident) => write!(f, "wait({})", wait_ident),
        }
    }
}
