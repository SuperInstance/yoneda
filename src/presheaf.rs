//! Presheaves and the presheaf category.
//!
//! A presheaf on C is a contravariant functor F: C^op → Set.
//! A representable presheaf is one of the form Hom(-, A) for some object A.
//!
//! The presheaf category has presheaves as objects and natural transformations
//! as morphisms.

use crate::hom_set::HomSet;
use crate::natural_transform::NaturalTransformation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A presheaf: a contravariant functor C^op → Set.
///
/// For each object X, stores F(X) as a set of strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presheaf {
    /// Name of this presheaf.
    pub name: String,
    /// Maps each object X to the elements of F(X).
    values: HashMap<String, Vec<String>>,
}

impl Presheaf {
    /// Create a new presheaf.
    pub fn new(name: impl Into<String>, values: HashMap<String, Vec<String>>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    /// Evaluate the presheaf at object X: returns the set F(X).
    pub fn apply(&self, x: &str) -> &[String] {
        self.values.get(x).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Size of F(X).
    pub fn size_at(&self, x: &str) -> usize {
        self.apply(x).len()
    }

    /// Objects this presheaf is defined on.
    pub fn objects(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }

    /// Construct a representable presheaf Hom(-, A) from the hom-set.
    pub fn representable(name: impl Into<String>, a: &str, hs: &HomSet) -> Self {
        let mut values = HashMap::new();
        for x in hs.objects() {
            let morphisms = hs.hom(x, a);
            let names: Vec<String> = morphisms.iter().map(|m| m.name.clone()).collect();
            values.insert(x.clone(), names);
        }
        Self::new(name, values)
    }

    /// Check if this presheaf is representable by comparing sizes with Hom(-, A).
    pub fn is_representable_by(&self, a: &str, hs: &HomSet) -> bool {
        for x in hs.objects() {
            if self.size_at(x) != hs.hom_size(x, a) {
                return false;
            }
        }
        true
    }

    /// Total size across all objects.
    pub fn total_size(&self) -> usize {
        self.values.values().map(|v| v.len()).sum()
    }

    /// Check if two presheaves are isomorphic (same sizes at all objects).
    pub fn isomorphic_to(&self, other: &Presheaf) -> bool {
        let all_keys: std::collections::HashSet<&String> =
            self.values.keys().chain(other.values.keys()).collect();
        for k in &all_keys {
            if self.size_at(k) != other.size_at(k) {
                return false;
            }
        }
        true
    }
}

/// The presheaf category: objects are presheaves, morphisms are natural transformations.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PresheafCategory {
    /// Presheaves in this category.
    presheaves: Vec<Presheaf>,
    /// Natural transformations between presheaves.
    transformations: Vec<NaturalTransformation>,
}

impl PresheafCategory {
    /// Create an empty presheaf category.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a presheaf.
    pub fn add_presheaf(&mut self, p: Presheaf) {
        self.presheaves.push(p);
    }

    /// Add a natural transformation.
    pub fn add_transformation(&mut self, t: NaturalTransformation) {
        self.transformations.push(t);
    }

    /// List all presheaves.
    pub fn presheaves(&self) -> &[Presheaf] {
        &self.presheaves
    }

    /// List all transformations.
    pub fn transformations(&self) -> &[NaturalTransformation] {
        &self.transformations
    }

    /// Find a presheaf by name.
    pub fn find_presheaf(&self, name: &str) -> Option<&Presheaf> {
        self.presheaves.iter().find(|p| p.name == name)
    }

    /// Find all transformations from presheaf `src` to presheaf `tgt`.
    pub fn hom(&self, src: &str, tgt: &str) -> Vec<&NaturalTransformation> {
        self.transformations
            .iter()
            .filter(|t| t.source_functor == src && t.target_functor == tgt)
            .collect()
    }

    /// Number of presheaves.
    pub fn num_presheaves(&self) -> usize {
        self.presheaves.len()
    }

    /// Number of transformations.
    pub fn num_transformations(&self) -> usize {
        self.transformations.len()
    }

    /// Construct the full representable presheaf category from a hom-set.
    /// Adds one representable presheaf y(A) = Hom(-, A) for each object A.
    pub fn from_hom_set(hs: &HomSet) -> Self {
        let mut cat = Self::new();
        for a in hs.objects() {
            let p = Presheaf::representable(format!("y({})", a), a, hs);
            cat.add_presheaf(p);
        }
        cat
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hom_set::Morphism;

    fn make_category() -> HomSet {
        let mut hs = HomSet::new(vec!["A".into(), "B".into()]);
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        hs.add_morphism(Morphism::new("f", "A", "B"));
        hs.add_morphism(Morphism::new("id_B", "B", "B"));
        hs.add_morphism(Morphism::new("g", "B", "A"));
        hs
    }

    #[test]
    fn test_presheaf_new() {
        let p = Presheaf::new(
            "P",
            HashMap::from([
                ("A".into(), vec!["x".into(), "y".into()]),
                ("B".into(), vec!["z".into()]),
            ]),
        );
        assert_eq!(p.size_at("A"), 2);
        assert_eq!(p.size_at("B"), 1);
        assert_eq!(p.size_at("C"), 0);
    }

    #[test]
    fn test_presheaf_total_size() {
        let p = Presheaf::new(
            "P",
            HashMap::from([
                ("A".into(), vec!["x".into()]),
                ("B".into(), vec!["y".into(), "z".into()]),
            ]),
        );
        assert_eq!(p.total_size(), 3);
    }

    #[test]
    fn test_representable_presheaf() {
        let hs = make_category();
        let p = Presheaf::representable("y(A)", "A", &hs);
        // Hom(-, A): A→A has id_A, B→A has g → sizes: A=1, B=1
        assert_eq!(p.size_at("A"), 1); // id_A
        assert_eq!(p.size_at("B"), 1); // g
    }

    #[test]
    fn test_representable_presheaf_b() {
        let hs = make_category();
        let p = Presheaf::representable("y(B)", "B", &hs);
        // Hom(-, B): A→B has f, B→B has id_B → sizes: A=1, B=1
        assert_eq!(p.size_at("A"), 1); // f
        assert_eq!(p.size_at("B"), 1); // id_B
    }

    #[test]
    fn test_is_representable_by() {
        let hs = make_category();
        let p = Presheaf::new(
            "P",
            HashMap::from([
                ("A".into(), vec!["id_A".into()]),
                ("B".into(), vec!["g".into()]),
            ]),
        );
        assert!(p.is_representable_by("A", &hs));
    }

    #[test]
    fn test_not_representable() {
        let hs = make_category();
        let p = Presheaf::new(
            "P",
            HashMap::from([
                ("A".into(), vec!["x".into(), "y".into(), "z".into()]),
                ("B".into(), vec!["w".into()]),
            ]),
        );
        assert!(!p.is_representable_by("A", &hs));
        assert!(!p.is_representable_by("B", &hs));
    }

    #[test]
    fn test_presheaf_isomorphic() {
        let p1 = Presheaf::new(
            "P1",
            HashMap::from([
                ("A".into(), vec!["x".into()]),
                ("B".into(), vec!["y".into()]),
            ]),
        );
        let p2 = Presheaf::new(
            "P2",
            HashMap::from([
                ("A".into(), vec!["a".into()]),
                ("B".into(), vec!["b".into()]),
            ]),
        );
        assert!(p1.isomorphic_to(&p2));
    }

    #[test]
    fn test_presheaf_not_isomorphic() {
        let p1 = Presheaf::new("P1", HashMap::from([("A".into(), vec!["x".into()])]));
        let p2 = Presheaf::new(
            "P2",
            HashMap::from([("A".into(), vec!["x".into(), "y".into()])]),
        );
        assert!(!p1.isomorphic_to(&p2));
    }

    #[test]
    fn test_presheaf_category_add() {
        let mut cat = PresheafCategory::new();
        cat.add_presheaf(Presheaf::new("P", HashMap::new()));
        cat.add_transformation(NaturalTransformation::identity("P", &[]));
        assert_eq!(cat.num_presheaves(), 1);
        assert_eq!(cat.num_transformations(), 1);
    }

    #[test]
    fn test_presheaf_category_find() {
        let mut cat = PresheafCategory::new();
        cat.add_presheaf(Presheaf::new("P1", HashMap::new()));
        cat.add_presheaf(Presheaf::new("P2", HashMap::new()));
        assert!(cat.find_presheaf("P1").is_some());
        assert!(cat.find_presheaf("P3").is_none());
    }

    #[test]
    fn test_presheaf_category_hom() {
        let mut cat = PresheafCategory::new();
        cat.add_transformation(NaturalTransformation::identity("F", &["A".into()]));
        cat.add_transformation(NaturalTransformation::new("eta", "F", "G", vec![]));
        let hom = cat.hom("F", "G");
        assert_eq!(hom.len(), 1);
        assert_eq!(hom[0].name, "eta");
    }

    #[test]
    fn test_from_hom_set() {
        let hs = make_category();
        let cat = PresheafCategory::from_hom_set(&hs);
        assert_eq!(cat.num_presheaves(), 2);
        assert!(cat.find_presheaf("y(A)").is_some());
        assert!(cat.find_presheaf("y(B)").is_some());
    }

    #[test]
    fn test_serialize_presheaf() {
        let p = Presheaf::new("P", HashMap::from([("A".into(), vec!["x".into()])]));
        let json = serde_json::to_string(&p).unwrap();
        let p2: Presheaf = serde_json::from_str(&json).unwrap();
        assert_eq!(p2.name, "P");
        assert_eq!(p2.size_at("A"), 1);
    }

    #[test]
    fn test_serialize_presheaf_category() {
        let mut cat = PresheafCategory::new();
        cat.add_presheaf(Presheaf::new("P", HashMap::new()));
        let json = serde_json::to_string(&cat).unwrap();
        let cat2: PresheafCategory = serde_json::from_str(&json).unwrap();
        assert_eq!(cat2.num_presheaves(), 1);
    }
}
