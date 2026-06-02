# delis + dryadsynth-bv — working notes

Delis learns logical formulas from finite models. It compiles a typed semantic
DSL down to SyGuS BV and races two backends: an internal closure-search and
either in-process `dryadsynth-bv` (sibling crate at `../dryadsynth-bv`) or an
external solver (`cvc5`). Sibling crates: `../dryadsynth-bv`, `../sygus-parser-rs`.

## Three load-bearing optimizations (don't undo)

These each give ~10× and compose. They apply uniformly to direct and LFP paths:

1. **Bisimulation quotient in delis** (`src/compile/bisim.rs`). Collapses
   equivalent aspects before BV encoding. Typical reduction: 80%+ bits
   (`random_52_tiny`: 5481 → 833).
2. **Backward relevance in dryadsynth-bv** (`exact/relevance/`). Drives
   projected dedup keys; ~4–20% of bits relevant on our problems.
3. **Let-binding ordering in delis-generated SyGuS** (`compile/compiler.rs`
   `BoolExpr::compact` / `BoolExpr::resolve_closure`). Lets the solver
   reuse subterms instead of re-evaluating.

## LFP path (parent semantics) — how it actually works

When the Delis semantic spec uses `(parent n)`, the compiler emits two
synth-funs: `g` (operator, real grammar) and `delis.closure.aux` (constant
grammar) joined by `(= g aux)`. This is the standard one-way translation of
two-way tree automata via least fixpoint (Vardi 1998 §4; Comon et al. 7.6.3).
No special "LFP design" — the LFP is just the math.

dryadsynth-bv detects this via `LfpSpec::detect_separate` (and the
`lfp-recursive` 1-synth-fun variant), then routes to the unified
`NullarySpec::search` with `fixpoint = Some(closure_name)`. There is no
parallel enumeration engine — both modes go through the same
`search_nullary_multi_nt`.

Three LFP-mode-specific behaviors inside that engine:

- **Sub-NT dedup uses a two-sample (`value@aux=0`, `value@aux=top`)
  fingerprint.** `aux=0` alone is biased — recursive sub-terms whose
  recursion doesn't fire at 0 collide with non-recursive ones and the
  synthesizer can't assemble recursive answers. The `top` sample lights
  up the recursion. Sound by monotone-lattice squeeze
  (`F(0) ≤ F(aux) ≤ F(top)`) for almost-all candidates; any false
  collision that drops a true LFP is caught downstream by the CEGIS
  verifier which re-checks under true LFP semantics. Implemented via
  `Candidate.value_top` and `pair_seen` in
  `dryadsynth-bv/src/exact.rs`. On `random_52_tiny` collapses 3.9M
  candidate pushes to 22.7k (~170×, ~10× wall).
- **DAG-eval in `materialize_and_kleene`.** Each candidate root walks the
  Node DAG via `eval_node_at_aux` directly instead of building a
  throwaway `OwnedExpr` via `materialize_node` per iter; `OwnedExpr` is
  materialized only for the winning candidate. Independent of dedup;
  ~4% wall on `random_52_tiny` (small because the walk cost dominates
  over Box allocation, which is what got eliminated).
- **Per-iter negative-label early-reject** in `materialize_and_kleene`:
  sound by monotonicity (`aux_n ≤ LFP`), saves Kleene iters on failing
  candidates.

Opt-in stats: set `DELIS_DEDUP_STATS=1` to print per-(nt, size) pushed /
distinct@0 / distinct@(0,top) counts at end of each LFP search round.
Useful for diagnosing whether the dedup is biting on new benchmarks.

## The memo cache (the LFP-path 6×)

`eval_defined_with_closure` in `dryadsynth-bv/src/exact.rs` wraps
`Environ::eval_defined` with a thread-local FxHashMap cache keyed on
`(fid, nargs, args)`. Enabled in `LfpSpec::search`, disabled in direct mode.

Empirically: 97–98% hit rate on solvable LFP cases. The same `(fid, args)`
tuple recurs ~40× across candidates and Kleene iters; the cache recovers
sub-NT-dedup-equivalent sharing lazily at the eval boundary. anbn-3 CEGIS:
~245ms → ~37ms with the cache.

## Hot-path gotchas

- **Cross-module `#[inline]` is not free.** Moving the memo cache from
  `exact.rs` to `closure/eval.rs` regressed random_52_tiny from ~35s to
  ~700s despite identical source. `#[inline(always)]` did not fix it.
  Keep `eval_defined_with_closure` and `EVAL_CACHE` in `exact.rs`.
- **`OwnedExpr::eval(args)` does NOT support closure binding.** Closure
  references in `define-fun` bodies are resolved via the `closure_args`
  metadata on each `DefineFun`, appended to the args list at eval time —
  not via `Var(0)` substitution. Use `eval_defined_with_closure` (or
  `OwnedExpr::eval_with_closure` for whole-tree walks).
- **`buf.clone()` for cache keys allocates per call.** Don't think a TLS
  args buffer eliminates allocation if the key still needs ownership.
- **First-run wall time on this MacBook is unreliable** — 2–10× variance
  is normal. Always do 3+ runs and trust steady-state.

## What didn't work (don't re-try without strong reason)

- Tree-walk cache keyed on `(structural_hash(OwnedExpr), aux)`: 0% hit rate.
  Each materialized candidate is structurally unique (different `Rhs × Rules`
  pair). Without hash-consing of the search's `Node` DAG, this can't fire.
- Split-key cache (`HashMap<(fid,nargs), HashMap<Box<[Value]>, Value>>`):
  doubled per-lookup HashMap probe → slower despite borrowed-slice lookups.
- Per-define-fun closure-bit-mask compressing cache keys: added BigUint
  `bitand` per lookup, outweighed the marginal cache-key compression
  (97% baseline hit rate left no headroom).
- `F(top)` upper-bound early-reject: one extra eval per candidate doesn't
  offset the savings on already-cheap Kleene.

## Coding preferences (Chentian)

- Function names should be short. `do_A_something` → `A.do_something`
  when it can naturally be a method.
- Modules should not know about concerns outside their responsibility.
  Example: `deffun.rs` (generic SyGuS define-fun bookkeeping) shouldn't
  use the word "closure" — that's LFP-search terminology.
- Prefer real algorithmic wins (bisim, relevance, let-binding) over
  Rust micro-tweaks (caches, hashing tricks). 1.5× tweaks aren't the
  goal; 10× restructuring is.
- "Think harder" means actually look at the data / read the code, not
  propose more options. Profile first, propose second.

## Useful commands

```bash
# rebuild both crates (delis depends on sibling dryadsynth-bv)
cargo build --release

# anbn-3 is the smallest LFP smoke test (~70ms wall after warmup)
time ./target/release/delis ./benchmark/CFG/anbn-3.sl

# random_52_tiny is the standing LFP performance reference. After the
# multi-sample sub-NT dedup landing: ~16s on remote (down from ~163s).
# A copy is checked into the repo root.
python3 benchmark/CFG/asp4cfg/scripts/gen.py \
  --data ~/Codes/Python/asp4cfg/datasets/random_52.txt \
  --k 3 --max-len-pos 5 --neg-cap 10 --out /tmp/random_52_tiny.sl
time ./target/release/delis ./random_52_tiny.sl --verbose

# LTL direct-mode regression check (~18-25s after warmup)
time ./target/release/delis benchmark/LTL/learnltl/syslite-comparison/glba_6802_1250.sl
```
