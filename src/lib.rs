pub mod backends;
pub mod compile;
pub mod lang;
pub mod parser;
pub mod process;
pub mod sorts;

/// Crate-wide hash-map/set/hasher aliases — the single point of control for
/// the hash algorithm used throughout delis. Every module imports
/// `HashMap`/`HashSet`/`FxHasher` from here instead of reaching for
/// `rustc_hash` directly, so swapping the hasher is a one-place edit.
///
/// Backed by rustc-hash's FxHash, chosen after benchmarking: on these
/// workloads the hot maps are integer-keyed (class ids, u64 cache keys,
/// dense-class tuples) where FxHash is optimal — an A/B against `foldhash`
/// came out neutral-to-slower (foldhash's edge is string keys, the minority
/// here).
pub mod fxmap {
    pub use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet, FxHasher};
}

pub use lang::*;
pub use parser::*;
