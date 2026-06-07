//! Hom-sets: morphisms between two objects in a concrete (Set) category.
//!
//! In category theory, `Hom(A, B)` is the set of all morphisms from object `A`
//! to object `B`. For our finite Set category, objects are `String` labels and
//! morphisms are named functions `String → String` with explicit source and
//! target annotations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A named morphism from one object to another.
///
/// In our concrete Set category, each morphism represents a function
/// `String → String` identified by name, with declared source and target objects.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Morphism {
    /// Human-readable name for this morphism.
    pub name: String,
    /// Source object in the category.
    pub source: String,
    /// Target object in the category.
    pub target: String,
}

impl Morphism {
    /// Create a new morphism.
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            source: source.into(),
            target: target.into(),
        }
    }

    /// Compose this morphism with another: `g ∘ f` where `self = f`, `other = g`.
    ///
    /// Returns `None` if the target of `self` does not match the source of `other`.
    pub fn compose(&self, other: &Morphism) -> Option<Morphism> {
        if self.target != other.source {
            return None;
        }
        Some(Morphism::new(
            format!("{}_∘_{}", other.name, self.name),
            &self.source,
            &other.target,
        ))
    }
}

/// A hom-set: the collection of morphisms between pairs of objects.
///
/// Internally stored as `HashMap<(source, target), Vec<Morphism>>`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HomSet {
    /// Objects in the category.
    objects: Vec<String>,
    /// Morphisms indexed by (source, target) pairs.
    morphisms: HashMap<String, Vec<Morphism>>,
}

impl HomSet {
    /// Create an empty hom-set with the given objects.
    pub fn new(objects: Vec<String>) -> Self {
        Self {
            objects,
            morphisms: HashMap::new(),
        }
    }

    /// Add an object to the category.
    pub fn add_object(&mut self, obj: impl Into<String>) {
        let o = obj.into();
        if !self.objects.contains(&o) {
            self.objects.push(o);
        }
    }

    /// Add a morphism to the hom-set.
    pub fn add_morphism(&mut self, m: Morphism) {
        self.add_object(&m.source);
        self.add_object(&m.target);
        let key = format!("{}→{}", m.source, m.target);
        self.morphisms.entry(key).or_default().push(m);
    }

    /// Get morphisms from `a` to `b`.
    pub fn hom(&self, a: &str, b: &str) -> Vec<&Morphism> {
        let key = format!("{}→{}", a, b);
        self.morphisms
            .get(&key)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }

    /// Get the number of morphisms from `a` to `b`.
    pub fn hom_size(&self, a: &str, b: &str) -> usize {
        self.hom(a, b).len()
    }

    /// List all objects.
    pub fn objects(&self) -> &[String] {
        &self.objects
    }

    /// List all morphisms.
    pub fn all_morphisms(&self) -> Vec<&Morphism> {
        self.morphisms.values().flatten().collect()
    }

    /// Compute the composition table: all valid compositions `g ∘ f`.
    ///
    /// Returns a vector of `(f, g, g∘f)` triples where composition is defined.
    pub fn composition_table(&self) -> Vec<(&Morphism, &Morphism, Morphism)> {
        let all: Vec<&Morphism> = self.all_morphisms();
        let mut table = Vec::new();
        for f in &all {
            for g in &all {
                if let Some(comp) = f.compose(g) {
                    table.push((*f, *g, comp));
                }
            }
        }
        table
    }

    /// Check if a morphism with the given name exists from `a` to `b`.
    pub fn has_morphism(&self, a: &str, b: &str, name: &str) -> bool {
        self.hom(a, b).iter().any(|m| m.name == name)
    }

    /// Identity morphism for an object.
    pub fn identity(&self, obj: &str) -> Morphism {
        Morphism::new(format!("id_{}", obj), obj, obj)
    }

    /// Verify that all identity morphisms exist (law check).
    pub fn verify_identities(&self) -> bool {
        for obj in &self.objects {
            let id = self.identity(obj);
            if !self.has_morphism(obj, obj, &id.name) {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_morphism_new() {
        let m = Morphism::new("f", "A", "B");
        assert_eq!(m.name, "f");
        assert_eq!(m.source, "A");
        assert_eq!(m.target, "B");
    }

    #[test]
    fn test_morphism_compose_valid() {
        let f = Morphism::new("f", "A", "B");
        let g = Morphism::new("g", "B", "C");
        let comp = f.compose(&g).unwrap();
        assert_eq!(comp.source, "A");
        assert_eq!(comp.target, "C");
        assert!(comp.name.contains("g"));
        assert!(comp.name.contains("f"));
    }

    #[test]
    fn test_morphism_compose_invalid() {
        let f = Morphism::new("f", "A", "B");
        let g = Morphism::new("g", "C", "D");
        assert!(f.compose(&g).is_none());
    }

    #[test]
    fn test_homset_add_and_query() {
        let mut hs = HomSet::new(vec!["A".into(), "B".into()]);
        hs.add_morphism(Morphism::new("f", "A", "B"));
        assert_eq!(hs.hom_size("A", "B"), 1);
        assert_eq!(hs.hom_size("B", "A"), 0);
    }

    #[test]
    fn test_homset_composition_table() {
        let mut hs = HomSet::new(vec!["A".into(), "B".into(), "C".into()]);
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        hs.add_morphism(Morphism::new("id_B", "B", "B"));
        hs.add_morphism(Morphism::new("f", "A", "B"));
        hs.add_morphism(Morphism::new("g", "B", "C"));
        let table = hs.composition_table();
        // g ∘ f should appear
        assert!(table.iter().any(|(f, g, _)| f.name == "f" && g.name == "g"));
    }

    #[test]
    fn test_homset_add_object_dedup() {
        let mut hs = HomSet::new(vec![]);
        hs.add_object("A");
        hs.add_object("A");
        hs.add_object("B");
        assert_eq!(hs.objects().len(), 2);
    }

    #[test]
    fn test_identity_morphism() {
        let hs = HomSet::new(vec!["X".into()]);
        let id = hs.identity("X");
        assert_eq!(id.source, "X");
        assert_eq!(id.target, "X");
    }

    #[test]
    fn test_verify_identities() {
        let mut hs = HomSet::new(vec!["A".into()]);
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        assert!(hs.verify_identities());
    }

    #[test]
    fn test_verify_identities_missing() {
        let hs = HomSet::new(vec!["A".into()]);
        assert!(!hs.verify_identities());
    }

    #[test]
    fn test_has_morphism() {
        let mut hs = HomSet::new(vec![]);
        hs.add_morphism(Morphism::new("f", "X", "Y"));
        assert!(hs.has_morphism("X", "Y", "f"));
        assert!(!hs.has_morphism("X", "Y", "g"));
    }

    #[test]
    fn test_all_morphisms() {
        let mut hs = HomSet::new(vec![]);
        hs.add_morphism(Morphism::new("f", "A", "B"));
        hs.add_morphism(Morphism::new("g", "B", "C"));
        assert_eq!(hs.all_morphisms().len(), 2);
    }

    #[test]
    fn test_serialize_deserialize_morphism() {
        let m = Morphism::new("f", "A", "B");
        let json = serde_json::to_string(&m).unwrap();
        let m2: Morphism = serde_json::from_str(&json).unwrap();
        assert_eq!(m, m2);
    }

    #[test]
    fn test_serialize_deserialize_homset() {
        let mut hs = HomSet::new(vec!["A".into()]);
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        let json = serde_json::to_string(&hs).unwrap();
        let hs2: HomSet = serde_json::from_str(&json).unwrap();
        assert_eq!(hs2.hom_size("A", "A"), 1);
    }
}
