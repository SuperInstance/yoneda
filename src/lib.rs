//! # Yoneda
//!
//! Yoneda lemma and representable functors for agent systems.
//!
//! This crate provides concrete implementations of category-theoretic
//! constructions over finite Set categories with `String` objects.

pub mod application;
pub mod hom_set;
pub mod natural_transform;
pub mod presheaf;
pub mod representable;
pub mod yoneda_embedding;

pub use application::{AgentProfile, CapabilityMatcher};
pub use hom_set::{HomSet, Morphism};
pub use natural_transform::NaturalTransformation;
pub use presheaf::{Presheaf, PresheafCategory};
pub use representable::{RepresentableFunctor, RepresentingObject};
pub use yoneda_embedding::YonedaEmbedding;
