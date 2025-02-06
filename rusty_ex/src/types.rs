use rustc_ast::NodeId;
use rustworkx_core::petgraph::dot::{Config, Dot};
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::Eq;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::panic;

// -------------------- Features --------------------

/// Simple feature
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub not: bool,
}

/// Complex feature: none, a single feature (not included), all features, or any feature
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ComplexFeature {
    None,
    Feature(Feature),
    All(Vec<ComplexFeature>),
    Any(Vec<ComplexFeature>),
}

// -------------------- Weights --------------------

/// Type of the weight of a node (not the actual weight, only the type)
/// The first String parameter of each variant is a debug string to identify the kind of item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeWeightKind {
    /// Item that has an intrinsic weight, like a literal or something that does NOT calls anything.
    /// The intrinsic weight is 1.0
    Intrinsic(String),
    /// Item which weight is determined by its children, like a block of statements, expressions or items.
    /// Has no intrinsic weight, the weight is the sum of the children
    Children(String),
    /// A reference to another item, like a function or a method call.
    /// The weight is the weight of the called thing, saved in the second parameter.
    Reference(String, Option<String>),
    /// Items that have no weight, like `_`.
    /// The weight is 0.0
    No(String),
}

/// Weight of a node: not yet calculated, a float, or waiting for something to be resolved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeWeight {
    ToBeCalculated,
    Weight(f64),
    Wait(String),
}

// -------------------- Graphs common --------------------

/// Edge between nodes, has a weight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub weight: f64,
}

// -------------------- AST Graph --------------------

/// Index of a node in the AST graph
pub type AstIndex = NodeIndex;

/// Key to uniquely identify an AST node (used to get the index in the `nodes` map)
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SimpleAstKey(pub NodeId);

/// SimpleASTKey, with support for multiple crates
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SuperAstKey {
    pub node_id: SimpleAstKey,
    pub krate: String,
}

/// Trait that identifies an AST key
pub trait AstKey: Hash + Eq + Clone + Debug + Display {}
impl AstKey for SimpleAstKey {}
impl AstKey for SuperAstKey {}

/// AST node, with features (complex, already parsed) and weight (kind and value).
/// The weight of an AST node is the sum of the weights of all its children plus
/// its intrinsic weight (if it has one).
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode<Key: AstKey> {
    pub node_id: Key,
    pub ident: Option<String>,
    pub features: ComplexFeature,
    pub weight_kind: NodeWeightKind,
    pub weight: NodeWeight,
}

/// AST graph: the actual graph and a map to get the index of a node from its key.
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstGraph<Key: AstKey> {
    pub graph: DiGraph<AstNode<Key>, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<Key, AstIndex>,
}

// -------------------- Features Graph --------------------

/// Index of a feature node in the features graph
pub type FeatureIndex = NodeIndex;

/// Key to uniquely identify a feature (used to get the index in the `nodes` map).
///
/// Features are shared between crates, so a Simple/Super key is not needed
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FeatureKey(pub Feature);

/// Feature node, with the feature and the weight (if calculated).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureNode {
    pub feature: FeatureKey,
    /// The weight is calculated "horizontally", considering only the "siblings" (this weight is the
    /// same weight that is on the outgoing edge to the parent node, it is duplicated for easier access).
    pub weight: Option<f64>,
    /// If this feature has some siblings, they must be satisfied (all), so we must track
    /// the nature of each single feature.
    pub complex_feature: Vec<ComplexFeature>,
}

/// Features graph: the actual graph and a map to get the index of a feature node from its key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesGraph {
    pub graph: DiGraph<FeatureNode, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<FeatureKey, FeatureIndex>,
}

// -------------------- Artifacts Graph --------------------

/// Index of an artifact node in the artifacts graph
pub type ArtifactIndex = NodeIndex;

/// Key to uniquely identify an artifact (used to get the index in the `nodes` map)
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SimpleArtifactKey(pub NodeId);

/// ArtifactKey, with support for multiple crates
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SuperArtifactKey {
    pub artifact: SimpleArtifactKey,
    pub krate: String,
}

/// Trait that identifies an AST node
pub trait ArtifactKey: Hash + Eq + Clone + Debug + Display {}
impl ArtifactKey for SimpleArtifactKey {}
impl ArtifactKey for SuperArtifactKey {}

/// Artifact node, with features (indexes in the features graph) and weight (sum of the weights
/// of all its children (even the not annotated ones) plus its intrinsic weight (if it has one)).
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactNode<Key: ArtifactKey> {
    pub artifact: Key,
    pub ident: Option<String>,
    pub complex_feature: ComplexFeature, // how the features are combined
    pub features_indexes: Vec<FeatureIndex>, // index in features graph
    pub weight: NodeWeight,
}

/// Artifacts graph: the actual graph and a map to get the index of an artifact node from its key.
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsGraph<Key: ArtifactKey> {
    pub graph: DiGraph<ArtifactNode<Key>, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<Key, ArtifactIndex>,
}

// -------------------- Serialization --------------------

/// Serialization of a single crate graphs
#[derive(Serialize, Deserialize)]
pub struct SimpleSerialization {
    pub ast_graph: AstGraph<SimpleAstKey>,
    pub features_graph: FeaturesGraph,
    pub artifacts_graph: ArtifactsGraph<SimpleArtifactKey>,
}

// -------------------- Implementations --------------------

impl Display for SimpleAstKey {
    /// SimpleAstKey to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SuperAstKey {
    /// SuperAstKey to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.krate, self.node_id)
    }
}

impl Display for SimpleArtifactKey {
    /// SimpleArtifactKey to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SuperArtifactKey {
    /// SuperArtifactKey to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.krate, self.artifact)
    }
}

impl<Key: AstKey> AstGraph<Key> {
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
        node_id: Key,
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
        let get_node_attr = |_g: &DiGraph<AstNode<Key>, Edge>, node: (NodeIndex, &AstNode<Key>)| {
            let index = node.0.index();
            let ast_node = node.1;
            format!(
                "label=\"i{}: node{} ({}) '{}' #[{}] {}\"",
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

impl<Key: AstKey> Default for AstGraph<Key> {
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
    pub fn create_node(
        &mut self,
        feature: FeatureKey,
        weight: Option<f64>,
        complex_feature: Vec<ComplexFeature>,
    ) -> FeatureIndex {
        if let Some(index) = self.nodes.get(&feature) {
            return *index;
        }

        let index = self.graph.add_node(FeatureNode {
            feature: feature.clone(),
            weight,
            complex_feature,
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
                true => format!(
                    "label=\"i{}: !{} [{}]\"",
                    index,
                    feature.feature.0.name,
                    feature
                        .complex_feature
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                false => format!(
                    "label=\"i{}: {} [{}]\"",
                    index,
                    feature.feature.0.name,
                    feature
                        .complex_feature
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                // true => format!("label=\"i{}: !{}\"", index, feature.feature.0.name),
                // false => format!("label=\"i{}: {}\"", index, feature.feature.0.name),
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

impl<Key: ArtifactKey> ArtifactsGraph<Key> {
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
        artifact: Key,
        ident: Option<String>,
        complex_feature: ComplexFeature,
        features_indexes: Vec<FeatureIndex>,
        weight: NodeWeight,
    ) -> ArtifactIndex {
        if let Some(index) = self.nodes.get(&artifact) {
            return *index;
        }

        let index = self.graph.add_node(ArtifactNode {
            artifact: artifact.clone(),
            ident,
            complex_feature,
            features_indexes,
            weight,
        });
        self.nodes.insert(artifact, index);

        index
    }

    /// Print artifacts graph in DOT format
    pub fn print_dot(&self) {
        let get_node_attr = |_g: &DiGraph<ArtifactNode<Key>, Edge>,
                             node: (NodeIndex, &ArtifactNode<Key>)| {
            let index = node.0.index();
            let artifact_node = node.1;
            format!(
                "label=\"i{} node{} '{}' [{}] {}\"",
                index,
                artifact_node.artifact,
                artifact_node.ident.clone().unwrap_or("-".to_string()),
                artifact_node.complex_feature,
                artifact_node.weight,
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

impl<Key: ArtifactKey> Default for ArtifactsGraph<Key> {
    fn default() -> Self {
        ArtifactsGraph::new()
    }
}

impl Default for SimpleAstKey {
    fn default() -> Self {
        panic!("Serialization error: default SimpleAstKey required")
    }
}

impl Default for SimpleArtifactKey {
    fn default() -> Self {
        panic!("Serialization error: default SimpleArtifactKey required")
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
            NodeWeightKind::Intrinsic(name) => write!(f, "Intrinsic({})", name),
            NodeWeightKind::Children(name) => write!(f, "Children({})", name),
            NodeWeightKind::Reference(name, ident) => write!(
                f,
                "Reference({})->{}",
                name,
                ident.clone().unwrap_or("??".to_string())
            ),
            NodeWeightKind::No(name) => write!(f, "No({})", name),
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

// -------------------- Simple Serialization --------------------

impl Serialize for SimpleAstKey {
    /// Serialize SimpleAstKey
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0.as_u32())
    }
}

impl<'de> Deserialize<'de> for SimpleAstKey {
    /// Deserialize SimpleAstKey
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(SimpleAstKey(NodeId::from_u32(value)))
    }
}

impl Serialize for SimpleArtifactKey {
    /// Serialize SimpleArtifactKey
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0.as_u32())
    }
}

impl<'de> Deserialize<'de> for SimpleArtifactKey {
    /// Deserialize SimpleArtifactKey
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(SimpleArtifactKey(NodeId::from_u32(value)))
    }
}
