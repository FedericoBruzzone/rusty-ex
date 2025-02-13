use petgraph::visit::IntoNodeReferences;
use rustc_ast::NodeId;
use rustworkx_core::petgraph::dot::{Config, Dot};
use rustworkx_core::petgraph::graph::{DiGraph, NodeIndex};
use serde::{Deserialize, Serialize};
use std::clone::Clone;
use std::cmp::Eq;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::panic;

use crate::configs::prop_formula::{PropFormula, ToPropFormula};

// Terminology:
// - Feature: an identifier that identifies a piece of code that can be included or excluded from compilation
// - Term: a piece of code that can be annotated by a feature (function, expression, literal, assignment, struct, ...)
// - Artifact: a term annotated by a feature

// -------------------- Features --------------------

/// Simple feature
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub not: bool,
}

/// Complex feature: none, a single feature (with not included), all features, or any feature
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ComplexFeature {
    None,
    Simple(Feature),
    All(Vec<ComplexFeature>),
    Any(Vec<ComplexFeature>),
}

// -------------------- Weights --------------------

/// Type of the weight of a term (or an artifact), not the actual weight, only the type
/// The first String parameter of each variant is a debug string to identify the kind of term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TermWeightKind {
    /// Term that has an intrinsic weight, like a literal or something that does NOT calls anything.
    /// The intrinsic weight is 1.0
    Intrinsic(String),
    /// Term which weight is determined by its children, like a block of statements, expressions or items.
    /// Has no intrinsic weight, the weight is the sum of the children
    Children(String),
    /// A reference to another term, like a function or a method call.
    /// The weight is the weight of the called thing, saved in the second parameter.
    Reference(String, Option<String>),
    /// Term that have no weight, like `_`.
    /// The weight is 0.0
    No(String),
}

/// Weight of a term (or an artifact): not yet calculated, a float, or waiting for something to be resolved
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TermWeight {
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

// -------------------- Terms Tree (Unified Intermediate Representation - UIR) --------------------

/// Key to uniquely identify a term node
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SimpleTermKey(pub NodeId);

/// Like SimpleTermKey, but with support for multiple crates
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SuperTermKey {
    pub node_id: SimpleTermKey,
    pub krate: String,
}

/// Trait that identifies a Term key
pub trait TermKey: Hash + Eq + Clone + Debug + Display {}
impl TermKey for SimpleTermKey {}
impl TermKey for SuperTermKey {}

/// Term node, with features (complex, already parsed) and weight (kind and value).
/// The weight of an term is defined by its kind and value.
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermNode<Key: TermKey> {
    pub node_id: Key,
    pub ident: Option<String>,
    pub features: ComplexFeature,
    pub weight_kind: TermWeightKind,
    pub weight: TermWeight,
}

/// Index of a term node in the graph representing the Terms Tree (UIR).
/// The index is used to access the node in the graph.
/// To get the index of a node from its key, use `TermsTree.nodes`
pub type TermIndex = NodeIndex;

/// Terms tree (UIR): the actual graph and a map to get the index of a node from its key.
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermsTree<Key: TermKey> {
    pub graph: DiGraph<TermNode<Key>, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<Key, TermIndex>,
}

// -------------------- Features Graph --------------------

/// Key to uniquely identify a feature.
///
/// Features are shared between crates, so a Simple/Super key is not needed
#[derive(Debug, Clone, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct FeatureKey(pub Feature);

impl From<&Feature> for FeatureKey {
    fn from(value: &Feature) -> Self {
        FeatureKey(value.clone())
    }
}

/// Feature node, with the feature, its weight (if already calculated) and its nature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureNode {
    pub feature: FeatureKey,
    /// The weight is calculated "horizontally", considering only the "siblings" (this weight is the
    /// same weight that is on the outgoing edge to the parent node, it is duplicated for easier access).
    pub weight: Option<f64>,
    /// If this feature has some siblings, they must be satisfied (all), so we must track
    /// the nature of each single feature.
    pub complex_feature: HashSet<ComplexFeature>,
}

/// Index of a feature node in the graph representing the Features Dependency Graph.
/// The index is used to access the node in the graph.
/// To get the index of a node from its key, use `FeaturesGraph.nodes`
pub type FeatureIndex = NodeIndex;

/// Features Dependency Graph: the actual graph and a map to get the index of a feature node from its key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesGraph {
    pub graph: DiGraph<FeatureNode, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<FeatureKey, FeatureIndex>,
}

// -------------------- Artifacts Tree --------------------

/// Key to uniquely identify an artifact
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SimpleArtifactKey(pub NodeId);

/// Liek ArtifactKey, but with support for multiple crates
#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub struct SuperArtifactKey {
    pub artifact: SimpleArtifactKey,
    pub krate: String,
}

/// Trait that identifies an artifact node
pub trait ArtifactKey: Hash + Eq + Clone + Debug + Display {}
impl ArtifactKey for SimpleArtifactKey {}
impl ArtifactKey for SuperArtifactKey {}

/// Artifact node, with features (indexes in the Features Graph) and weight (the weight
/// of the Term that annotated becomes an artifact).
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactNode<Key: ArtifactKey> {
    pub artifact: Key,
    pub ident: Option<String>,
    /// Feature that annotate the Term, making it an Artifact
    pub complex_feature: ComplexFeature,
    /// Indexes of the features that compose the complex feature in the Features Graph
    pub features_indexes: Vec<FeatureIndex>,
    pub weight: TermWeight,
}

/// Index of a artifact node in the graph representing the Artifacts Dependency Tree.
/// The index is used to access the node in the graph.
/// To get the index of a node from its key, use `ArtifactsTree.nodes`
pub type ArtifactIndex = NodeIndex;

/// Artifacts tree: the actual graph and a map to get the index of an artifact node from its key.
///
/// Generic over the key type, to support both Simple (single crate) and Super (multiple crates) keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsTree<Key: ArtifactKey> {
    pub graph: DiGraph<ArtifactNode<Key>, Edge>,
    #[serde(skip)]
    pub nodes: HashMap<Key, ArtifactIndex>,
}

// -------------------- Serialization --------------------

/// Serialization of a single crate graphs
#[derive(Serialize, Deserialize)]
pub struct SimpleSerialization {
    pub terms_tree: TermsTree<SimpleTermKey>,
    pub features_graph: FeaturesGraph,
    pub artifacts_tree: ArtifactsTree<SimpleArtifactKey>,
}

// -------------------- Implementations --------------------

impl Display for SimpleTermKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SuperTermKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.krate, self.node_id)
    }
}

impl Display for SimpleArtifactKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for SuperArtifactKey {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}::{}", self.krate, self.artifact)
    }
}

impl Feature {
    pub fn is_negated(&self) -> bool {
        self.not
    }
}

impl<Key: TermKey> TermsTree<Key> {
    /// Create a new empty Terms Tree (UIR)
    pub fn new() -> Self {
        TermsTree {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create a term node in the Terms Tree and add it to the terms nodes hashmap.
    /// Return the index of the created node
    pub fn create_node(
        &mut self,
        node_id: Key,
        ident: Option<String>,
        features: ComplexFeature,
        weight_kind: TermWeightKind,
        weight: TermWeight,
    ) -> TermIndex {
        if let Some(index) = self.nodes.get(&node_id) {
            return *index;
        }

        let index = self.graph.add_node(TermNode {
            node_id: node_id.clone(),
            ident,
            features,
            weight_kind,
            weight,
        });
        self.nodes.insert(node_id, index);

        index
    }

    /// Print Terms Tree (UIR) in DOT format
    pub fn print_dot(&self) {
        let get_node_attr = |_g: &DiGraph<TermNode<Key>, Edge>,
                             node: (NodeIndex, &TermNode<Key>)| {
            let index = node.0.index();
            let term_node = node.1;
            format!(
                "label=\"i{}: node{} ({}) '{}' #[{}] {}\"",
                index,
                term_node.node_id,
                term_node.weight_kind,
                term_node.ident.clone().unwrap_or(" ".to_string()),
                term_node.features,
                term_node.weight,
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

impl<Key: TermKey> Default for TermsTree<Key> {
    fn default() -> Self {
        TermsTree::new()
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
        complex_feature: HashSet<ComplexFeature>,
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

impl ToPropFormula<Feature> for FeaturesGraph {
    fn to_prop_formula(&self) -> PropFormula<Feature> {
        fn resolve_complex_feature_rec(complex_feature: &ComplexFeature) -> PropFormula<Feature> {
            match complex_feature {
                ComplexFeature::None => PropFormula::None,
                ComplexFeature::Simple(feature) => {
                    if feature.is_negated() {
                        PropFormula::Not(Box::new(PropFormula::Var(feature.clone())))
                    } else {
                        PropFormula::Var(feature.clone())
                    }
                }
                ComplexFeature::All(features) => {
                    let mut formula = Vec::new();
                    for feature in features {
                        formula.push(resolve_complex_feature_rec(feature));
                    }
                    PropFormula::And(formula)
                }
                ComplexFeature::Any(features) => {
                    let mut formula = Vec::new();
                    for feature in features {
                        formula.push(resolve_complex_feature_rec(feature));
                    }
                    PropFormula::Or(formula)
                }
            }
        }

        let mut formula = Vec::new();
        for (_, feature_node) in self.graph.node_references() {
            for complex_feature in &feature_node.complex_feature {
                formula.push(resolve_complex_feature_rec(complex_feature));
            }
        }

        PropFormula::And(formula)
    }
}

impl<Key: ArtifactKey> ArtifactsTree<Key> {
    /// Create a new empty artifacts tree
    pub fn new() -> Self {
        ArtifactsTree {
            graph: DiGraph::new(),
            nodes: HashMap::new(),
        }
    }

    /// Create an artifact node in the artifacts tree and add it to the artifacts nodes hashmap.
    /// Return the index of the created node
    pub fn create_node(
        &mut self,
        artifact: Key,
        ident: Option<String>,
        complex_feature: ComplexFeature,
        features_indexes: Vec<FeatureIndex>,
        weight: TermWeight,
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

    /// Print artifacts tree in DOT format
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

impl<Key: ArtifactKey> Default for ArtifactsTree<Key> {
    fn default() -> Self {
        ArtifactsTree::new()
    }
}

impl Default for SimpleTermKey {
    fn default() -> Self {
        panic!("Serialization error: default SimpleTermKey required")
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
            ComplexFeature::Simple(Feature { name, not }) => {
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

impl TermWeightKind {
    /// Parse the kind variant name from the debug string (keep only the Kind name)
    pub fn parse_kind_variant_name(s: String) -> String {
        s.split(['(', '{']).next().unwrap_or("").trim().to_string()
    }
}

impl Display for TermWeightKind {
    /// NodeWeightKind to string
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TermWeightKind::Intrinsic(name) => write!(f, "Intrinsic({})", name),
            TermWeightKind::Children(name) => write!(f, "Children({})", name),
            TermWeightKind::Reference(name, ident) => write!(
                f,
                "Reference({})->{}",
                name,
                ident.clone().unwrap_or("??".to_string())
            ),
            TermWeightKind::No(name) => write!(f, "No({})", name),
        }
    }
}

impl Display for TermWeight {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TermWeight::ToBeCalculated => write!(f, "w-"),
            TermWeight::Weight(w) => write!(f, "w{:.2}", w),
            TermWeight::Wait(wait_ident) => write!(f, "wait({})", wait_ident),
        }
    }
}

// -------------------- Simple Serialization --------------------

impl Serialize for SimpleTermKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0.as_u32())
    }
}

impl<'de> Deserialize<'de> for SimpleTermKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(SimpleTermKey(NodeId::from_u32(value)))
    }
}

impl Serialize for SimpleArtifactKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.0.as_u32())
    }
}

impl<'de> Deserialize<'de> for SimpleArtifactKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u32::deserialize(deserializer)?;
        Ok(SimpleArtifactKey(NodeId::from_u32(value)))
    }
}
