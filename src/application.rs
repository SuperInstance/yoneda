//! Agent capabilities as presheaves.
//!
//! This module applies the Yoneda lemma to agent systems:
//! - Agent profiles are presheaves over a capability category
//! - Capability matching is natural transformation search
//! - Similarity scores via the Yoneda lemma

use crate::hom_set::{HomSet, Morphism};
use crate::natural_transform::{NatComponent, NaturalTransformation};
use crate::presheaf::{Presheaf, PresheafCategory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An agent profile: a presheaf over the capability category.
///
/// Each capability X maps to the agent's proficiency set F(X) — the set of
/// ways the agent can exercise capability X.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    /// Agent name / identifier.
    pub name: String,
    /// The underlying presheaf.
    pub presheaf: Presheaf,
}

impl AgentProfile {
    /// Create a new agent profile.
    pub fn new(name: impl Into<String>, capabilities: HashMap<String, Vec<String>>) -> Self {
        let name_str = name.into();
        Self {
            name: name_str.clone(),
            presheaf: Presheaf::new(name_str, capabilities),
        }
    }

    /// Get the agent's capabilities at a given skill level.
    pub fn capability_at(&self, x: &str) -> &[String] {
        self.presheaf.apply(x)
    }

    /// Number of distinct capabilities at object X.
    pub fn capability_size(&self, x: &str) -> usize {
        self.presheaf.size_at(x)
    }

    /// Total capability breadth.
    pub fn total_capabilities(&self) -> usize {
        self.presheaf.total_size()
    }

    /// List capability domains this agent operates in.
    pub fn capability_domains(&self) -> Vec<&str> {
        self.presheaf.objects()
    }
}

/// Capability matcher: uses natural transformations and Yoneda for matching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatcher {
    /// The capability category (hom-set).
    pub category: HomSet,
    /// Agent profiles indexed by name.
    agents: HashMap<String, AgentProfile>,
}

impl CapabilityMatcher {
    /// Create a new capability matcher with the given category.
    pub fn new(category: HomSet) -> Self {
        Self {
            category,
            agents: HashMap::new(),
        }
    }

    /// Register an agent profile.
    pub fn register(&mut self, agent: AgentProfile) {
        self.agents.insert(agent.name.clone(), agent);
    }

    /// Get a registered agent.
    pub fn get_agent(&self, name: &str) -> Option<&AgentProfile> {
        self.agents.get(name)
    }

    /// List all registered agents.
    pub fn agents(&self) -> Vec<&AgentProfile> {
        self.agents.values().collect()
    }

    /// Check if an agent has a specific capability.
    pub fn has_capability(&self, agent: &str, capability: &str, item: &str) -> bool {
        self.agents
            .get(agent)
            .map(|a| a.capability_at(capability).contains(&item.to_string()))
            .unwrap_or(false)
    }

    /// Compute the Yoneda similarity score between two agents.
    ///
    /// By the Yoneda lemma, an agent is determined by its relationships to all
    /// other agents. We compute similarity as the overlap in capability profiles.
    ///
    /// Returns a value in [0.0, 1.0].
    pub fn yoneda_similarity(&self, a: &str, b: &str) -> f64 {
        let agent_a = match self.agents.get(a) {
            Some(a) => a,
            None => return 0.0,
        };
        let agent_b = match self.agents.get(b) {
            Some(b) => b,
            None => return 0.0,
        };

        let all_domains: std::collections::HashSet<&str> = agent_a
            .presheaf
            .objects()
            .into_iter()
            .chain(agent_b.presheaf.objects())
            .collect();

        if all_domains.is_empty() {
            return 1.0; // both empty → identical
        }

        let mut total = 0usize;
        let mut matching = 0usize;

        for domain in &all_domains {
            let set_a: std::collections::HashSet<&String> =
                agent_a.presheaf.apply(domain).iter().collect();
            let set_b: std::collections::HashSet<&String> =
                agent_b.presheaf.apply(domain).iter().collect();
            total += set_a.len().max(set_b.len());
            matching += set_a.intersection(&set_b).count();
        }

        if total == 0 {
            return 1.0;
        }
        matching as f64 / total as f64
    }

    /// Find the best natural transformation from agent A's profile to a target profile.
    ///
    /// Returns the name of the best-matching agent and the similarity score.
    pub fn best_match(&self, query: &str) -> Option<(String, f64)> {
        let mut best: Option<(String, f64)> = None;
        for agent in self.agents.values() {
            if agent.name == query {
                continue;
            }
            let score = self.yoneda_similarity(query, &agent.name);
            match best {
                None => best = Some((agent.name.clone(), score)),
                Some((_, best_score)) if score > best_score => {
                    best = Some((agent.name.clone(), score));
                }
                _ => {}
            }
        }
        best
    }

    /// Check if a natural transformation exists from agent A to agent B.
    ///
    /// A natural transformation exists if for every capability domain X,
    /// there is a mapping from A(X) to B(X).
    pub fn has_natural_transform(&self, a: &str, b: &str) -> bool {
        let agent_a = match self.agents.get(a) {
            Some(a) => a,
            None => return false,
        };
        let agent_b = match self.agents.get(b) {
            Some(b) => b,
            None => return false,
        };

        // Every element in A's capabilities maps to something in B's
        for domain in agent_a.capability_domains() {
            let b_caps = agent_b.capability_at(domain);
            if b_caps.is_empty() {
                return false;
            }
        }
        true
    }

    /// Construct the natural transformation components from agent A to agent B.
    pub fn make_nat_transform(&self, a: &str, b: &str) -> Option<NaturalTransformation> {
        if !self.has_natural_transform(a, b) {
            return None;
        }
        let agent_a = self.agents.get(a)?;
        let components: Vec<NatComponent> = agent_a
            .capability_domains()
            .iter()
            .map(|domain| NatComponent::new(*domain, format!("{}_to_{}_{}", a, b, domain)))
            .collect();
        Some(NaturalTransformation::new(
            format!("{}→{}", a, b),
            a,
            b,
            components,
        ))
    }

    /// Build a presheaf category from registered agents.
    pub fn to_presheaf_category(&self) -> PresheafCategory {
        let mut cat = PresheafCategory::new();
        for agent in self.agents.values() {
            cat.add_presheaf(agent.presheaf.clone());
        }
        cat
    }
}

/// Build a capability category from capability names.
pub fn build_capability_category(capabilities: &[&str]) -> HomSet {
    let mut hs = HomSet::new(capabilities.iter().map(|s| s.to_string()).collect());
    for cap in capabilities {
        hs.add_morphism(Morphism::new(format!("id_{}", cap), *cap, *cap));
    }
    hs
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_matcher() -> CapabilityMatcher {
        let hs = build_capability_category(&["coding", "writing", "math"]);
        let mut matcher = CapabilityMatcher::new(hs);
        matcher.register(AgentProfile::new(
            "alice",
            HashMap::from([
                ("coding".into(), vec!["rust".into(), "python".into()]),
                ("writing".into(), vec!["docs".into()]),
                ("math".into(), vec!["algebra".into()]),
            ]),
        ));
        matcher.register(AgentProfile::new(
            "bob",
            HashMap::from([
                ("coding".into(), vec!["rust".into(), "js".into()]),
                ("writing".into(), vec!["blog".into()]),
                ("math".into(), vec!["calculus".into()]),
            ]),
        ));
        matcher.register(AgentProfile::new(
            "carol",
            HashMap::from([
                ("coding".into(), vec!["python".into(), "js".into()]),
                ("writing".into(), vec!["docs".into(), "blog".into()]),
                ("math".into(), vec!["algebra".into(), "calculus".into()]),
            ]),
        ));
        matcher
    }

    #[test]
    fn test_agent_profile() {
        let alice = AgentProfile::new(
            "alice",
            HashMap::from([("coding".into(), vec!["rust".into()])]),
        );
        assert_eq!(alice.capability_size("coding"), 1);
        assert_eq!(alice.capability_size("writing"), 0);
        assert_eq!(alice.total_capabilities(), 1);
    }

    #[test]
    fn test_capability_domains() {
        let agent = AgentProfile::new(
            "x",
            HashMap::from([
                ("a".into(), vec!["1".into()]),
                ("b".into(), vec!["2".into()]),
            ]),
        );
        let mut domains = agent.capability_domains();
        domains.sort();
        assert_eq!(domains, vec!["a", "b"]);
    }

    #[test]
    fn test_register_and_get() {
        let matcher = make_matcher();
        assert!(matcher.get_agent("alice").is_some());
        assert!(matcher.get_agent("dave").is_none());
    }

    #[test]
    fn test_has_capability() {
        let matcher = make_matcher();
        assert!(matcher.has_capability("alice", "coding", "rust"));
        assert!(!matcher.has_capability("alice", "coding", "js"));
        assert!(!matcher.has_capability("dave", "coding", "rust"));
    }

    #[test]
    fn test_yoneda_similarity_self() {
        let matcher = make_matcher();
        let score = matcher.yoneda_similarity("alice", "alice");
        assert!((score - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_yoneda_similarity_partial() {
        let matcher = make_matcher();
        let score = matcher.yoneda_similarity("alice", "bob");
        assert!(score > 0.0);
        assert!(score < 1.0);
    }

    #[test]
    fn test_yoneda_similarity_unknown() {
        let matcher = make_matcher();
        assert_eq!(matcher.yoneda_similarity("alice", "nobody"), 0.0);
        assert_eq!(matcher.yoneda_similarity("nobody", "alice"), 0.0);
    }

    #[test]
    fn test_best_match() {
        let matcher = make_matcher();
        let best = matcher.best_match("alice");
        assert!(best.is_some());
        let (name, score) = best.unwrap();
        assert_ne!(name, "alice");
        assert!(score > 0.0);
    }

    #[test]
    fn test_has_natural_transform() {
        let matcher = make_matcher();
        // alice → carol: carol has at least one item in every domain
        assert!(matcher.has_natural_transform("alice", "carol"));
    }

    #[test]
    fn test_make_nat_transform() {
        let matcher = make_matcher();
        let nt = matcher.make_nat_transform("alice", "carol");
        assert!(nt.is_some());
        let nt = nt.unwrap();
        assert_eq!(nt.source_functor, "alice");
        assert_eq!(nt.target_functor, "carol");
        assert_eq!(nt.len(), 3); // coding, writing, math
    }

    #[test]
    fn test_build_capability_category() {
        let hs = build_capability_category(&["a", "b"]);
        assert_eq!(hs.objects().len(), 2);
        assert_eq!(hs.hom_size("a", "a"), 1); // id_a
    }

    #[test]
    fn test_to_presheaf_category() {
        let matcher = make_matcher();
        let cat = matcher.to_presheaf_category();
        assert_eq!(cat.num_presheaves(), 3);
    }

    #[test]
    fn test_agents_list() {
        let matcher = make_matcher();
        assert_eq!(matcher.agents().len(), 3);
    }

    #[test]
    fn test_serialize_agent_profile() {
        let agent = AgentProfile::new("test", HashMap::from([("x".into(), vec!["a".into()])]));
        let json = serde_json::to_string(&agent).unwrap();
        let a2: AgentProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(a2.name, "test");
        assert_eq!(a2.capability_size("x"), 1);
    }

    #[test]
    fn test_serialize_matcher() {
        let matcher = make_matcher();
        let json = serde_json::to_string(&matcher).unwrap();
        let m2: CapabilityMatcher = serde_json::from_str(&json).unwrap();
        assert!(m2.get_agent("alice").is_some());
    }

    #[test]
    fn test_similarity_symmetric() {
        let matcher = make_matcher();
        let s1 = matcher.yoneda_similarity("alice", "bob");
        let s2 = matcher.yoneda_similarity("bob", "alice");
        assert!((s1 - s2).abs() < 0.001);
    }
}
