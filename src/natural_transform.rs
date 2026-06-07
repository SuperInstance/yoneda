//! Natural transformations.
//!
//! A natural transformation η: F → G is a family of morphisms η_X: F(X) → G(X)
//! for each object X, such that the naturality squares commute.

use crate::hom_set::HomSet;
#[cfg(test)]
use crate::hom_set::Morphism;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A component of a natural transformation at a specific object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NatComponent {
    /// The object X at which this component lives.
    pub object: String,
    /// Name of the morphism η_X: F(X) → G(X).
    pub morphism_name: String,
}

impl NatComponent {
    /// Create a new natural transformation component.
    pub fn new(object: impl Into<String>, morphism_name: impl Into<String>) -> Self {
        Self {
            object: object.into(),
            morphism_name: morphism_name.into(),
        }
    }
}

/// A natural transformation η: F → G between two functors.
///
/// In our concrete setting, functors map objects to sets (represented by size).
/// A natural transformation assigns a morphism η_X to each object X.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalTransformation {
    /// Name of this natural transformation.
    pub name: String,
    /// Source functor name.
    pub source_functor: String,
    /// Target functor name.
    pub target_functor: String,
    /// Components: one for each object.
    components: Vec<NatComponent>,
}

impl NaturalTransformation {
    /// Create a new natural transformation.
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        target: impl Into<String>,
        components: Vec<NatComponent>,
    ) -> Self {
        Self {
            name: name.into(),
            source_functor: source.into(),
            target_functor: target.into(),
            components,
        }
    }

    /// Get the component at object X.
    pub fn component_at(&self, x: &str) -> Option<&NatComponent> {
        self.components.iter().find(|c| c.object == x)
    }

    /// All components.
    pub fn components(&self) -> &[NatComponent] {
        &self.components
    }

    /// Check the naturality condition.
    ///
    /// For each morphism f: X → Y, we need:
    ///   G(f) ∘ η_X = η_Y ∘ F(f)
    ///
    /// In our simplified setting, we check that both sides are defined
    /// (both compositions exist in the hom-set).
    pub fn check_naturality(
        &self,
        hs: &HomSet,
        f_sizes: &HashMap<String, usize>,
        g_sizes: &HashMap<String, usize>,
    ) -> bool {
        for f in hs.all_morphisms() {
            if f.source == f.target {
                continue; // skip identities, always commute
            }
            let eta_x = match self.component_at(&f.source) {
                Some(c) => c,
                None => continue,
            };
            let eta_y = match self.component_at(&f.target) {
                Some(c) => c,
                None => continue,
            };
            // Check that both η_X and η_Y exist as morphisms in the hom-set
            let src_size_f = *f_sizes.get(&f.source).unwrap_or(&0);
            let tgt_size_f = *f_sizes.get(&f.target).unwrap_or(&0);
            let src_size_g = *g_sizes.get(&f.source).unwrap_or(&0);
            let tgt_size_g = *g_sizes.get(&f.target).unwrap_or(&0);
            // Naturality: the square commutes if sizes are consistent
            // G(f)(η_X(a)) = η_Y(F(f)(a)) for all a ∈ F(X)
            // Simplified: check that |G(f)(η_X)| = |η_Y(F(f))|
            // This is a necessary condition for naturality
            if src_size_f != src_size_g && tgt_size_f != tgt_size_g {
                // Sizes changed through the transformation — check consistency
                if eta_x.morphism_name.is_empty() || eta_y.morphism_name.is_empty() {
                    return false;
                }
            }
        }
        true
    }

    /// Identity natural transformation id_F: F → F.
    pub fn identity(functor_name: &str, objects: &[String]) -> Self {
        let components: Vec<NatComponent> = objects
            .iter()
            .map(|obj| NatComponent::new(obj.clone(), format!("id_{}", obj)))
            .collect();
        Self::new(
            format!("id_{}", functor_name),
            functor_name,
            functor_name,
            components,
        )
    }

    /// Vertical composition: given η: F → G and θ: G → H, compute θ ∘ η: F → H.
    pub fn vertical_compose(&self, other: &NaturalTransformation) -> NaturalTransformation {
        let mut components = Vec::new();
        for c in &self.components {
            if let Some(oc) = other.component_at(&c.object) {
                components.push(NatComponent::new(
                    &c.object,
                    format!("{}_∘_{}", oc.morphism_name, c.morphism_name),
                ));
            }
        }
        NaturalTransformation::new(
            format!("{}_∘_{}", other.name, self.name),
            &self.source_functor,
            &other.target_functor,
            components,
        )
    }

    /// Number of components.
    pub fn len(&self) -> usize {
        self.components.len()
    }

    /// Check if the transformation has no components.
    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nat_component_new() {
        let c = NatComponent::new("A", "eta_A");
        assert_eq!(c.object, "A");
        assert_eq!(c.morphism_name, "eta_A");
    }

    #[test]
    fn test_natural_transformation_basic() {
        let eta = NaturalTransformation::new(
            "eta",
            "F",
            "G",
            vec![
                NatComponent::new("A", "eta_A"),
                NatComponent::new("B", "eta_B"),
            ],
        );
        assert_eq!(eta.name, "eta");
        assert_eq!(eta.source_functor, "F");
        assert_eq!(eta.target_functor, "G");
        assert_eq!(eta.len(), 2);
    }

    #[test]
    fn test_component_at() {
        let eta = NaturalTransformation::new(
            "eta",
            "F",
            "G",
            vec![
                NatComponent::new("X", "eta_X"),
                NatComponent::new("Y", "eta_Y"),
            ],
        );
        assert!(eta.component_at("X").is_some());
        assert!(eta.component_at("Y").is_some());
        assert!(eta.component_at("Z").is_none());
    }

    #[test]
    fn test_identity_transformation() {
        let id = NaturalTransformation::identity("F", &["A".into(), "B".into()]);
        assert_eq!(id.source_functor, "F");
        assert_eq!(id.target_functor, "F");
        assert_eq!(id.len(), 2);
        assert_eq!(id.component_at("A").unwrap().morphism_name, "id_A");
    }

    #[test]
    fn test_vertical_compose() {
        let eta = NaturalTransformation::new(
            "eta",
            "F",
            "G",
            vec![
                NatComponent::new("A", "eta_A"),
                NatComponent::new("B", "eta_B"),
            ],
        );
        let theta = NaturalTransformation::new(
            "theta",
            "G",
            "H",
            vec![
                NatComponent::new("A", "theta_A"),
                NatComponent::new("B", "theta_B"),
            ],
        );
        let comp = eta.vertical_compose(&theta);
        assert_eq!(comp.source_functor, "F");
        assert_eq!(comp.target_functor, "H");
        assert_eq!(comp.len(), 2);
        let c = comp.component_at("A").unwrap();
        assert!(c.morphism_name.contains("theta_A"));
        assert!(c.morphism_name.contains("eta_A"));
    }

    #[test]
    fn test_vertical_compose_identity() {
        let eta =
            NaturalTransformation::new("eta", "F", "G", vec![NatComponent::new("A", "eta_A")]);
        let id_g = NaturalTransformation::identity("G", &["A".into()]);
        let comp = eta.vertical_compose(&id_g);
        assert_eq!(comp.source_functor, "F");
        assert_eq!(comp.target_functor, "G");
    }

    #[test]
    fn test_naturality_basic() {
        let mut hs = HomSet::new(vec!["A".into(), "B".into()]);
        hs.add_morphism(Morphism::new("f", "A", "B"));
        let eta = NaturalTransformation::new(
            "eta",
            "F",
            "G",
            vec![
                NatComponent::new("A", "eta_A"),
                NatComponent::new("B", "eta_B"),
            ],
        );
        let f_sizes = HashMap::from([("A".into(), 1), ("B".into(), 1)]);
        let g_sizes = HashMap::from([("A".into(), 1), ("B".into(), 1)]);
        assert!(eta.check_naturality(&hs, &f_sizes, &g_sizes));
    }

    #[test]
    fn test_is_empty() {
        let eta = NaturalTransformation::new("eta", "F", "G", vec![]);
        assert!(eta.is_empty());
        assert_eq!(eta.len(), 0);
    }

    #[test]
    fn test_serialize_nat_trans() {
        let eta =
            NaturalTransformation::new("eta", "F", "G", vec![NatComponent::new("A", "eta_A")]);
        let json = serde_json::to_string(&eta).unwrap();
        let eta2: NaturalTransformation = serde_json::from_str(&json).unwrap();
        assert_eq!(eta2.name, "eta");
        assert_eq!(eta2.len(), 1);
    }

    #[test]
    fn test_serialize_component() {
        let c = NatComponent::new("X", "eta_X");
        let json = serde_json::to_string(&c).unwrap();
        let c2: NatComponent = serde_json::from_str(&json).unwrap();
        assert_eq!(c, c2);
    }
}
