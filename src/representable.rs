//! Representable functors.
//!
//! A functor `F` is *representable* if there exists an object `A` such that
//! `F(X) ≅ Hom(A, X)` for all objects `X`. The object `A` is called the
//! *representing object*.

use crate::hom_set::HomSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A concrete functor from our finite Set category to Set.
///
/// For each object X, stores `F(X)` as a set of strings (the image).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentableFunctor {
    /// Name of this functor.
    pub name: String,
    /// Maps each object X to the size |F(X)|.
    values: HashMap<String, usize>,
}

impl RepresentableFunctor {
    /// Create a new functor with the given mapping.
    pub fn new(name: impl Into<String>, values: HashMap<String, usize>) -> Self {
        Self {
            name: name.into(),
            values,
        }
    }

    /// Evaluate the functor at object X: returns |F(X)|.
    pub fn apply(&self, x: &str) -> usize {
        *self.values.get(x).unwrap_or(&0)
    }

    /// The objects this functor is defined on.
    pub fn objects(&self) -> Vec<&str> {
        self.values.keys().map(|s| s.as_str()).collect()
    }

    /// Check if this functor is representable in the given hom-set category.
    ///
    /// We check: does there exist an object A such that |F(X)| = |Hom(A, X)|
    /// for all X?
    pub fn is_representable(&self, hs: &HomSet) -> bool {
        self.find_representing_object(hs).is_some()
    }

    /// Find the representing object A such that F(X) ≅ Hom(A, X).
    pub fn find_representing_object(&self, hs: &HomSet) -> Option<String> {
        for a in hs.objects() {
            let mut matches = true;
            for x in hs.objects() {
                if self.apply(x) != hs.hom_size(a, x) {
                    matches = false;
                    break;
                }
            }
            if matches {
                return Some(a.clone());
            }
        }
        None
    }

    /// Total size of the functor's image across all objects.
    pub fn total_size(&self) -> usize {
        self.values.values().sum()
    }
}

/// A representing object: the object A that represents a functor F via
/// the natural isomorphism F(X) ≅ Hom(A, X).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepresentingObject {
    /// The representing object's name.
    pub object: String,
    /// The functor it represents.
    pub functor_name: String,
}

impl RepresentingObject {
    /// Create a new representing object.
    pub fn new(object: impl Into<String>, functor_name: impl Into<String>) -> Self {
        Self {
            object: object.into(),
            functor_name: functor_name.into(),
        }
    }

    /// Verify that this object actually represents the given functor.
    pub fn verify(&self, functor: &RepresentableFunctor, hs: &HomSet) -> bool {
        for x in hs.objects() {
            if functor.apply(x) != hs.hom_size(&self.object, x) {
                return false;
            }
        }
        true
    }

    /// Construct the natural isomorphism entries: for each X, list the
    /// correspondence F(X) ↔ Hom(A, X).
    pub fn isomorphism_entries(
        &self,
        functor: &RepresentableFunctor,
        hs: &HomSet,
    ) -> Vec<(String, usize, usize)> {
        hs.objects()
            .iter()
            .map(|x| {
                let f_val = functor.apply(x);
                let hom_val = hs.hom_size(&self.object, x);
                (x.clone(), f_val, hom_val)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hom_set::Morphism;

    fn make_category() -> HomSet {
        let mut hs = HomSet::new(vec!["A".into(), "B".into(), "C".into()]);
        // Hom(A,A)=2, Hom(A,B)=1, Hom(A,C)=0
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        hs.add_morphism(Morphism::new("f_AA", "A", "A"));
        hs.add_morphism(Morphism::new("f_AB", "A", "B"));
        // Hom(B,B)=1, Hom(B,A)=0, Hom(B,C)=1
        hs.add_morphism(Morphism::new("id_B", "B", "B"));
        hs.add_morphism(Morphism::new("g_BC", "B", "C"));
        // Hom(C,C)=1
        hs.add_morphism(Morphism::new("id_C", "C", "C"));
        // A→C: 0, B→A: 0, C→A: 0, C→B: 0
        hs
    }

    #[test]
    fn test_functor_apply() {
        let f = RepresentableFunctor::new(
            "F",
            HashMap::from([("A".into(), 2), ("B".into(), 1), ("C".into(), 0)]),
        );
        assert_eq!(f.apply("A"), 2);
        assert_eq!(f.apply("B"), 1);
        assert_eq!(f.apply("C"), 0);
        assert_eq!(f.apply("Z"), 0); // unknown object
    }

    #[test]
    fn test_representable_yes() {
        let hs = make_category();
        // Hom(A, -) has sizes: A=2, B=1, C=0
        let f = RepresentableFunctor::new(
            "F_A",
            HashMap::from([("A".into(), 2), ("B".into(), 1), ("C".into(), 0)]),
        );
        assert!(f.is_representable(&hs));
        assert_eq!(f.find_representing_object(&hs), Some("A".to_string()));
    }

    #[test]
    fn test_representable_no() {
        let hs = make_category();
        // No object has this hom profile
        let f = RepresentableFunctor::new(
            "Bad",
            HashMap::from([("A".into(), 5), ("B".into(), 5), ("C".into(), 5)]),
        );
        assert!(!f.is_representable(&hs));
    }

    #[test]
    fn test_representing_object_verify() {
        let hs = make_category();
        let f = RepresentableFunctor::new(
            "F_A",
            HashMap::from([("A".into(), 2), ("B".into(), 1), ("C".into(), 0)]),
        );
        let ro = RepresentingObject::new("A", "F_A");
        assert!(ro.verify(&f, &hs));
    }

    #[test]
    fn test_representing_object_verify_fail() {
        let hs = make_category();
        let f = RepresentableFunctor::new(
            "F",
            HashMap::from([("A".into(), 5), ("B".into(), 5), ("C".into(), 5)]),
        );
        let ro = RepresentingObject::new("A", "F");
        assert!(!ro.verify(&f, &hs));
    }

    #[test]
    fn test_isomorphism_entries() {
        let hs = make_category();
        let f = RepresentableFunctor::new(
            "F_A",
            HashMap::from([("A".into(), 2), ("B".into(), 1), ("C".into(), 0)]),
        );
        let ro = RepresentingObject::new("A", "F_A");
        let entries = ro.isomorphism_entries(&f, &hs);
        // Each entry should have equal functor and hom values
        for (_, fv, hv) in &entries {
            assert_eq!(fv, hv);
        }
    }

    #[test]
    fn test_total_size() {
        let f = RepresentableFunctor::new(
            "F",
            HashMap::from([("A".into(), 3), ("B".into(), 2), ("C".into(), 1)]),
        );
        assert_eq!(f.total_size(), 6);
    }

    #[test]
    fn test_functor_objects() {
        let f = RepresentableFunctor::new("F", HashMap::from([("X".into(), 1), ("Y".into(), 2)]));
        let mut objs = f.objects();
        objs.sort();
        assert_eq!(objs, vec!["X", "Y"]);
    }

    #[test]
    fn test_serialize_functor() {
        let f = RepresentableFunctor::new("F", HashMap::from([("A".into(), 1)]));
        let json = serde_json::to_string(&f).unwrap();
        let f2: RepresentableFunctor = serde_json::from_str(&json).unwrap();
        assert_eq!(f2.name, "F");
        assert_eq!(f2.apply("A"), 1);
    }

    #[test]
    fn test_representable_by_b() {
        let hs = make_category();
        // Hom(B, -) has sizes: A=0, B=1, C=1
        let f = RepresentableFunctor::new(
            "F_B",
            HashMap::from([("A".into(), 0), ("B".into(), 1), ("C".into(), 1)]),
        );
        assert!(f.is_representable(&hs));
        assert_eq!(f.find_representing_object(&hs), Some("B".to_string()));
    }
}
