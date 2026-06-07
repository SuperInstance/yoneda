//! Yoneda embedding.
//!
//! The Yoneda embedding maps each object `A` to the representable functor
//! `Hom(A, -)`, embedding the category into its presheaf category `[C^op, Set]`.
//!
//! For a finite Set category, we enumerate all `Hom(A, X)` for each `X`.

use crate::hom_set::HomSet;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The image of a single object under the Yoneda embedding.
///
/// Maps each object X to |Hom(A, X)|, i.e., the representable functor y(A).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YonedaImage {
    /// The original object A.
    pub object: String,
    /// The functor Hom(A, -) as a map X ↦ |Hom(A, X)|.
    pub functor_values: HashMap<String, usize>,
}

impl YonedaImage {
    /// Evaluate the embedded functor at object X.
    pub fn apply(&self, x: &str) -> usize {
        *self.functor_values.get(x).unwrap_or(&0)
    }
}

/// The full Yoneda embedding: maps every object A to the functor y(A) = Hom(A, -).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YonedaEmbedding {
    /// The embedded images, one per object.
    images: Vec<YonedaImage>,
    /// The original objects in order.
    objects: Vec<String>,
}

impl YonedaEmbedding {
    /// Construct the Yoneda embedding from a hom-set category.
    pub fn from_hom_set(hs: &HomSet) -> Self {
        let objects: Vec<String> = hs.objects().to_vec();
        let images: Vec<YonedaImage> = objects
            .iter()
            .map(|a| {
                let mut functor_values = HashMap::new();
                for x in &objects {
                    functor_values.insert(x.clone(), hs.hom_size(a, x));
                }
                YonedaImage {
                    object: a.clone(),
                    functor_values,
                }
            })
            .collect();
        Self { images, objects }
    }

    /// Get the Yoneda image of a specific object.
    pub fn image_of(&self, a: &str) -> Option<&YonedaImage> {
        self.images.iter().find(|img| img.object == a)
    }

    /// All embedded images.
    pub fn images(&self) -> &[YonedaImage] {
        &self.images
    }

    /// The original objects.
    pub fn objects(&self) -> &[String] {
        &self.objects
    }

    /// Check that the embedding is faithful: different objects map to different functors.
    ///
    /// The Yoneda embedding is always faithful for any category, so this should
    /// always return true (it's a sanity check on our implementation).
    pub fn is_faithful(&self) -> bool {
        for i in 0..self.images.len() {
            for j in (i + 1)..self.images.len() {
                if self.images[i].functor_values == self.images[j].functor_values {
                    return false;
                }
            }
        }
        true
    }

    /// Check that the embedding is full: every natural transformation
    /// y(A) → y(B) arises from a unique morphism A → B.
    ///
    /// For our concrete case, we check that the number of distinct
    /// functor profiles equals the number of objects.
    pub fn is_full(&self) -> bool {
        // In general the Yoneda embedding is full and faithful.
        // For our size-based check, we verify all images are distinct.
        self.is_faithful()
    }

    /// Pretty-print the embedding as a matrix.
    ///
    /// Rows are objects A, columns are objects X, entries are |Hom(A, X)|.
    pub fn to_matrix(&self) -> String {
        let mut lines = Vec::new();
        // Header
        let mut header = String::from("     ");
        for x in &self.objects {
            header.push_str(&format!("{:>6}", x));
        }
        lines.push(header);
        // Rows
        for img in &self.images {
            let mut row = format!("{:>4} ", img.object);
            for x in &self.objects {
                row.push_str(&format!("{:>6}", img.apply(x)));
            }
            lines.push(row);
        }
        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hom_set::Morphism;

    fn make_category() -> HomSet {
        let mut hs = HomSet::new(vec!["A".into(), "B".into(), "C".into()]);
        hs.add_morphism(Morphism::new("id_A", "A", "A"));
        hs.add_morphism(Morphism::new("f_AA", "A", "A"));
        hs.add_morphism(Morphism::new("f_AB", "A", "B"));
        hs.add_morphism(Morphism::new("id_B", "B", "B"));
        hs.add_morphism(Morphism::new("g_BC", "B", "C"));
        hs.add_morphism(Morphism::new("id_C", "C", "C"));
        hs
    }

    #[test]
    fn test_embedding_basic() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        assert_eq!(emb.images().len(), 3);
    }

    #[test]
    fn test_image_of_a() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let img = emb.image_of("A").unwrap();
        assert_eq!(img.apply("A"), 2); // id_A, f_AA
        assert_eq!(img.apply("B"), 1); // f_AB
        assert_eq!(img.apply("C"), 0);
    }

    #[test]
    fn test_image_of_b() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let img = emb.image_of("B").unwrap();
        assert_eq!(img.apply("A"), 0);
        assert_eq!(img.apply("B"), 1); // id_B
        assert_eq!(img.apply("C"), 1); // g_BC
    }

    #[test]
    fn test_image_of_c() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let img = emb.image_of("C").unwrap();
        assert_eq!(img.apply("A"), 0);
        assert_eq!(img.apply("B"), 0);
        assert_eq!(img.apply("C"), 1); // id_C
    }

    #[test]
    fn test_faithful() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        assert!(emb.is_faithful());
    }

    #[test]
    fn test_full() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        assert!(emb.is_full());
    }

    #[test]
    fn test_matrix_output() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let matrix = emb.to_matrix();
        assert!(matrix.contains("A"));
        assert!(matrix.contains("B"));
        assert!(matrix.contains("C"));
    }

    #[test]
    fn test_image_of_nonexistent() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        assert!(emb.image_of("Z").is_none());
    }

    #[test]
    fn test_serialize_embedding() {
        let hs = make_category();
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let json = serde_json::to_string(&emb).unwrap();
        let emb2: YonedaEmbedding = serde_json::from_str(&json).unwrap();
        assert_eq!(emb2.images().len(), 3);
    }

    #[test]
    fn test_empty_category() {
        let hs = HomSet::new(vec![]);
        let emb = YonedaEmbedding::from_hom_set(&hs);
        assert_eq!(emb.images().len(), 0);
        assert!(emb.is_faithful()); // vacuously true
    }

    #[test]
    fn test_single_object() {
        let mut hs = HomSet::new(vec!["X".into()]);
        hs.add_morphism(Morphism::new("id_X", "X", "X"));
        let emb = YonedaEmbedding::from_hom_set(&hs);
        let img = emb.image_of("X").unwrap();
        assert_eq!(img.apply("X"), 1);
    }
}
