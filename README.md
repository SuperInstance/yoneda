# yoneda

**Yoneda lemma and representable functors for agent systems.**

A concrete Rust implementation of fundamental category-theoretic constructions—hom-sets, representable functors, the Yoneda embedding, natural transformations, presheaves, and their application to agent capability matching—all over finite `Set` categories with `String` objects.

> "The Yoneda lemma is arguably the most important result in category theory."
> — Emily Riehl, *Category Theory in Context* (2016)

---

## Table of Contents

- [Motivation](#motivation)
- [Theory](#theory)
  - [The Yoneda Lemma](#the-yoneda-lemma)
  - [Proof Sketch](#proof-sketch)
  - [Representable Functors](#representable-functors)
  - [The Yoneda Embedding](#the-yoneda-embedding)
  - [Presheaves](#presheaves)
- [Architecture](#architecture)
- [Module Reference](#module-reference)
- [Design Decisions](#design-decisions)
- [Quick Start](#quick-start)
- [Examples](#examples)
  - [Example 1: Building a Category and Checking Representability](#example-1-building-a-category-and-checking-representability)
  - [Example 2: The Yoneda Embedding](#example-2-the-yoneda-embedding)
  - [Example 3: Agent Capability Matching](#example-3-agent-capability-matching)
- [ASCII Art: Yoneda Embedding Diagram](#ascii-art-yoneda-embedding-diagram)
- [API Overview](#api-overview)
- [Testing](#testing)
- [Benchmarks](#benchmarks)
- [References](#references)
- [License](#license)

---

## Motivation

Category theory provides a powerful language for structured relationships. The Yoneda lemma, in particular, tells us that an object is completely determined by its relationships to all other objects. This insight is directly applicable to agent systems:

- **Agent profiles** can be modeled as presheaves over capability categories
- **Capability matching** becomes a natural transformation search problem
- **Similarity** is quantified through Yoneda-style profiling

This crate makes these abstract ideas concrete by working entirely with `String` objects and `HashMap`-backed hom-sets. No generic category traits, no higher-kinded types—just data structures and algorithms you can use today.

---

## Theory

### The Yoneda Lemma

**Statement (Covariant Yoneda Lemma).** Let **C** be a locally small category, `A` an object of **C**, and `F: C → Set` a functor. Then there is a bijection:

```
Nat(Hom(A, -), F) ≅ F(A)
```

which is natural in both `A` and `F`.

In words: natural transformations from the representable functor `Hom(A, -)` to any functor `F` are in bijection with elements of `F(A)`.

**Statement (Contravariant Yoneda Lemma).** Dually:

```
Nat(Hom(-, A), F) ≅ F(A)
```

for a contravariant functor (presheaf) `F: C^op → Set`.

### Proof Sketch

The proof is remarkably elegant:

1. **Construction (⇒):** Given η ∈ Nat(Hom(A, -), F), define the element `x = η_A(id_A) ∈ F(A)`.

2. **Construction (⇐):** Given `x ∈ F(A)`, define for each `f: A → B`:
   `η_B(f) = F(f)(x)`.

3. **Naturality:** The naturality square for `g: B → C`:
   ```
         η_B
   Hom(A,B) → F(B)
     |          |
   Hom(A,g)   F(g)
     |          |
     v    η_C   v
   Hom(A,C) → F(C)
   ```
   commutes because `η_C(g ∘ f) = F(g ∘ f)(x) = F(g)(F(f)(x)) = F(g)(η_B(f))`.

4. **Inverse:** These two constructions are inverses. Starting with `x`, going forward gives `η_A(id_A) = F(id_A)(x) = x`. Starting with `η`, going forward then back recovers `η` by naturality.

For the full proof, see Mac Lane (1971, §III.2) or Riehl (2016, §2.2).

### Representable Functors

A functor `F: C → Set` is **representable** if there exists an object `A` of `C` and a natural isomorphism:

```
F ≅ Hom(A, -)
```

The object `A` is called the **representing object**. By the Yoneda lemma, the representing object (if it exists) is unique up to isomorphism.

In this crate, we check representability by comparing sizes: `F(X) ≅ Hom(A, X)` means `|F(X)| = |Hom(A, X)|` for all objects `X`.

### The Yoneda Embedding

The **Yoneda embedding** is the functor:

```
y: C → [C^op, Set]
 A ↦ Hom(-, A)
```

This embedding is:
- **Faithful:** `y` is injective on morphisms
- **Full:** every natural transformation `y(A) → y(B)` comes from a morphism `A → B`

Hence **C** embeds fully and faithfully into its presheaf category `[C^op, Set]`.

### Presheaves

A **presheaf** on **C** is a contravariant functor `F: C^op → Set`. The category of presheaves `[C^op, Set]` has:
- **Objects:** presheaves `F, G, ...`
- **Morphisms:** natural transformations `η: F → G`

A **representable presheaf** is one of the form `Hom(-, A)` for some object `A`. The Yoneda lemma tells us these are the "atomic" building blocks of the presheaf category.

---

## Architecture

```
yoneda
├── hom_set           Morphisms and hom-sets
├── representable     Representable functors and representing objects
├── yoneda_embedding  The Yoneda embedding y: C → [C^op, Set]
├── natural_transform Natural transformations: components, composition, identity
├── presheaf          Presheaves and the presheaf category
└── application       Agent capability matching via Yoneda
```

Each module is self-contained with no cross-dependencies beyond `hom_set`.

---

## Module Reference

| Module | Key Types | Description |
|--------|-----------|-------------|
| `hom_set` | `Morphism`, `HomSet` | Morphisms between `String` objects; composition tables |
| `representable` | `RepresentableFunctor`, `RepresentingObject` | Check if `F ≅ Hom(A, -)` |
| `yoneda_embedding` | `YonedaEmbedding`, `YonedaImage` | Embed `C` into `[C^op, Set]` |
| `natural_transform` | `NaturalTransformation`, `NatComponent` | Family of morphisms `η_X: F(X) → G(X)` |
| `presheaf` | `Presheaf`, `PresheafCategory` | Contravariant functors `C^op → Set` |
| `application` | `AgentProfile`, `CapabilityMatcher` | Agent capabilities as presheaves |

---

## Design Decisions

1. **Concrete over abstract.** No generic `Category` trait. Objects are `String`, hom-sets are `HashMap<String, Vec<Morphism>>`. This keeps the code accessible and debuggable.

2. **Size-based representability.** We check `F ≅ Hom(A, -)` by comparing set sizes rather than constructing explicit bijections. This is sufficient for finite categories and avoids the complexity of function representation.

3. **Serde everywhere.** All public types derive `Serialize + Deserialize`. Agent profiles, hom-sets, and matchers can be persisted to JSON and restored.

4. **Zero external deps (except serde).** No `num-traits`, no `frunk`, no category theory libraries. The implementation is self-contained.

5. **Agent-first application.** The `application` module isn't an afterthought—it's the motivation. Real agent capability matching through category theory.

---

## Quick Start

```toml
[dependencies]
yoneda = "0.1"
```

```rust
use yoneda::hom_set::{HomSet, Morphism};
use yoneda::application::{AgentProfile, CapabilityMatcher, build_capability_category};
use std::collections::HashMap;

fn main() {
    // Build a capability category
    let cat = build_capability_category(&["code", "write", "think"]);

    // Create agent profiles (presheaves over the capability category)
    let mut matcher = CapabilityMatcher::new(cat);
    matcher.register(AgentProfile::new("bot_a", HashMap::from([
        ("code".into(), vec!["rust".into(), "python".into()]),
        ("write".into(), vec!["docs".into()]),
    ])));
    matcher.register(AgentProfile::new("bot_b", HashMap::from([
        ("code".into(), vec!["python".into()]),
        ("think".into(), vec!["reason".into()]),
    ])));

    // Compute Yoneda similarity
    let score = matcher.yoneda_similarity("bot_a", "bot_b");
    println!("Similarity: {:.2}", score);
}
```

---

## Examples

### Example 1: Building a Category and Checking Representability

```rust
use yoneda::hom_set::{HomSet, Morphism};
use yoneda::representable::RepresentableFunctor;
use std::collections::HashMap;

// Create a small category with objects A, B, C
let mut cat = HomSet::new(vec!["A".into(), "B".into(), "C".into()]);

// Add morphisms
cat.add_morphism(Morphism::new("id_A", "A", "A"));
cat.add_morphism(Morphism::new("id_B", "B", "B"));
cat.add_morphism(Morphism::new("id_C", "C", "C"));
cat.add_morphism(Morphism::new("f", "A", "B"));
cat.add_morphism(Morphism::new("g", "B", "C"));
cat.add_morphism(Morphism::new("h", "A", "C"));

// Hom(A, -) has sizes: A=1, B=1, C=1
let functor = RepresentableFunctor::new("F", HashMap::from([
    ("A".into(), 1),
    ("B".into(), 1),
    ("C".into(), 1),
]));

// Is F representable?
if functor.is_representable(&cat) {
    let repr = functor.find_representing_object(&cat).unwrap();
    println!("F is represented by: {}", repr);
}
```

### Example 2: The Yoneda Embedding

```rust
use yoneda::hom_set::{HomSet, Morphism};
use yoneda::yoneda_embedding::YonedaEmbedding;

let mut cat = HomSet::new(vec!["X".into(), "Y".into()]);
cat.add_morphism(Morphism::new("id_X", "X", "X"));
cat.add_morphism(Morphism::new("id_Y", "Y", "Y"));
cat.add_morphism(Morphism::new("f", "X", "Y"));
cat.add_morphism(Morphism::new("g", "Y", "X"));

// Construct the Yoneda embedding
let emb = YonedaEmbedding::from_hom_set(&cat);

// Each object maps to a representable functor
for image in emb.images() {
    println!("y({}): {:?}", image.object, image.functor_values);
}

// The embedding is full and faithful
assert!(emb.is_faithful());
assert!(emb.is_full());

// Pretty-print the matrix
println!("{}", emb.to_matrix());
```

### Example 3: Agent Capability Matching

```rust
use yoneda::application::{AgentProfile, CapabilityMatcher, build_capability_category};
use std::collections::HashMap;

let cat = build_capability_category(&["search", "code", "analyze"]);
let mut matcher = CapabilityMatcher::new(cat);

// Register agents as presheaves
matcher.register(AgentProfile::new("research_agent", HashMap::from([
    ("search".into(), vec!["web".into(), "paper".into()]),
    ("analyze".into(), vec!["summarize".into()]),
])));
matcher.register(AgentProfile::new("code_agent", HashMap::from([
    ("code".into(), vec!["rust".into(), "python".into()]),
    ("analyze".into(), vec!["review".into()]),
])));
matcher.register(AgentProfile::new("full_agent", HashMap::from([
    ("search".into(), vec!["web".into()]),
    ("code".into(), vec!["python".into()]),
    ("analyze".into(), vec!["summarize".into(), "review".into()]),
])));

// Find the best match for research_agent
let (best_name, score) = matcher.best_match("research_agent").unwrap();
println!("Best match: {} (score: {:.2})", best_name, score);

// Check natural transformation existence
if matcher.has_natural_transform("research_agent", "full_agent") {
    let nt = matcher.make_nat_transform("research_agent", "full_agent").unwrap();
    println!("Natural transformation: {} → {}", nt.source_functor, nt.target_functor);
    for comp in nt.components() {
        println!("  η_{} = {}", comp.object, comp.morphism_name);
    }
}
```

---

## ASCII Art: Yoneda Embedding Diagram

```
          THE YONEDA EMBEDDING
          y: C ────────────▶ [C^op, Set]

   C (Objects)                Presheaf Category
   ─────────────              ──────────────────

        A        ──────────▶  y(A) = Hom(-, A)
       / \                      │
      /   \                     │ F(X) = { f | f: X → A }
     f     g                    │ for each X in C
    /       \                   │
   v         v                  │ A presheaf: maps each
  B    ──────▶  y(B) = Hom(-, B)   object X to Hom(X, B)
   \       /                   │
    \     /                    │ Natural transformations
     h   k                     │ are the morphisms
      \ /                      │
       v                       v
        C    ──────────▶  y(C) = Hom(-, C)


   ════════════════════════════════════════════════

   YONEDA LEMMA (Covariant):
   
   Nat(Hom(A, -), F)  ≅  F(A)
   
   "Natural transformations from a representable
    functor to F are in bijection with F(A)."

   ════════════════════════════════════════════════

   COMMUTING NATURALITY SQUARE:

       Hom(A, B) ──η_B──▶ F(B)
          │                │
    Hom(A,f)              F(f)
          │                │
          ▼       η_C      ▼
       Hom(A, C) ────────▶ F(C)

   For all f: B → C, the square commutes:
   F(f) ∘ η_B = η_C ∘ Hom(A, f)

   ════════════════════════════════════════════════

   AGENT AS PRESHEAF:

   Agent Profile P: Capability^op → Set
   
   P("code")   = { "rust", "python" }
   P("write")  = { "docs" }
   P("think")  = { "reason", "plan" }
   
   Capability matching = finding a natural
   transformation η: P₁ → P₂ between agent
   presheaves. Yoneda similarity quantifies
   how well profiles align.
```

---

## API Overview

### `hom_set`

```rust
// Create a morphism
let f = Morphism::new("f", "A", "B");

// Build a hom-set
let mut hs = HomSet::new(vec!["A".into(), "B".into()]);
hs.add_morphism(f);

// Query
let morphisms = hs.hom("A", "B");  // Vec<&Morphism>
let count = hs.hom_size("A", "B"); // usize
let table = hs.composition_table(); // Vec<(&Morphism, &Morphism, Morphism)>
```

### `representable`

```rust
let functor = RepresentableFunctor::new("F", values_map);
let is_repr = functor.is_representable(&hom_set);
let repr = functor.find_representing_object(&hom_set); // Option<String>

let ro = RepresentingObject::new("A", "F");
assert!(ro.verify(&functor, &hom_set));
```

### `yoneda_embedding`

```rust
let emb = YonedaEmbedding::from_hom_set(&hs);
let img = emb.image_of("A"); // Option<&YonedaImage>
emb.is_faithful(); // bool
emb.is_full();     // bool
println!("{}", emb.to_matrix()); // Pretty matrix
```

### `natural_transform`

```rust
let eta = NaturalTransformation::new("eta", "F", "G", components);
let id = NaturalTransformation::identity("F", &objects);
let composed = eta.vertical_compose(&theta);
eta.check_naturality(&hs, &f_sizes, &g_sizes);
```

### `presheaf`

```rust
let p = Presheaf::new("P", values);
let repr = Presheaf::representable("y(A)", "A", &hs);
p.is_representable_by("A", &hs);

let mut cat = PresheafCategory::new();
cat.add_presheaf(p);
cat = PresheafCategory::from_hom_set(&hs); // Auto-generate y(A) for each A
```

### `application`

```rust
let agent = AgentProfile::new("name", capabilities);
let matcher = CapabilityMatcher::new(category);
matcher.register(agent);
matcher.yoneda_similarity("a", "b"); // f64 in [0, 1]
matcher.best_match("query"); // Option<(String, f64)>
matcher.has_natural_transform("a", "b"); // bool
```

---

## Testing

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific module tests
cargo test hom_set
cargo test representable
cargo test yoneda_embedding
cargo test natural_transform
cargo test presheaf
cargo test application
```

The test suite includes **74 tests** covering:
- Morphism creation, composition, and invalid composition
- Hom-set construction, querying, and composition tables
- Representability checking (positive and negative cases)
- Yoneda embedding faithfulness and fullness
- Natural transformation identity, vertical composition, and naturality
- Presheaf construction, representability, and isomorphism checking
- Agent profile creation and capability matching
- Yoneda similarity computation (self-similarity, partial, unknown agents)
- Serialization/deserialization round-trips for all public types

---

## Benchmarks

This crate operates over finite categories with `String` objects. Performance characteristics:

- **Hom-set lookup:** O(n) where n = number of morphisms in the hom-set (stored as `Vec`)
- **Representability check:** O(k²) where k = number of objects (check each candidate)
- **Yoneda embedding:** O(k²) — enumerate all pairs
- **Similarity:** O(k × m) where k = number of capability domains, m = max set size

For production use with large categories, consider replacing `Vec` with `HashSet` for hom-sets.

---

## References

1. **Mac Lane, Saunders.** *Categories for the Working Mathematician.* 2nd ed., Springer, 1971. — The foundational text; Yoneda lemma in §III.2.

2. **Riehl, Emily.** *Category Theory in Context.* Dover, 2016. — Modern introduction with excellent exposition of the Yoneda lemma in §2.2.

3. **Awodey, Steve.** *Category Theory.* 2nd ed., Oxford University Press, 2010. — Accessible introduction; Yoneda lemma in Chapter 2.

4. **Loregian, Fosco.** *(Co)end Calculus.* Cambridge University Press, 2021. — Advanced treatment showing how the Yoneda lemma underlies end/coend calculus.

5. **Leinster, Tom.** *Basic Category Theory.* Cambridge University Press, 2014. — Gentle introduction with clear proofs of the Yoneda lemma.

6. **Borceux, Francis.** *Handbook of Categorical Algebra, Vol. 1.* Cambridge University Press, 1994. — Comprehensive reference; Yoneda lemma and representable functors in §1.9.

7. **nLab contributors.** "Yoneda lemma." *nLab*, 2024. — Online reference with connections to higher category theory. https://ncatlab.org/nlab/show/Yoneda+lemma

---

## License

MIT
