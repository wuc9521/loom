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

## Multi-nonterminal CFG search: why it's hard, and what's intrinsic

Context: the K>=2 CFG benchmarks (`benchmark/CFG/sipser/`, generated by
`asp4cfg/scripts/gen.py` via `gen_sipser.py`) learn grammars with several
nonterminals S0..S{k-1}. Two separate things were investigated; keep them
straight.

### Encoding fix (real bug, fixed)

`gen.py`'s `Rules.Sem` clause originally only climbed upward on `reset.Si`
at a `cfg.lhs.Si` node. The paper's reset(z) `_` rule (Krogmeier &
Madhusudan 2023, Fig 7) requires climbing for *any* reset mode regardless
of the node's label. Without it, a `reset.Sj` (j!=i) walk dies the moment
it crosses a `cfg.lhs.Si` node with a different label, so the rules spine
cannot mix nonterminals and every multi-NT solution is silently rejected.
Fixed by using the `all_reset_or` disjunction (same as `Rhs.Sem` already
did). This was necessary but NOT sufficient to see multi-NT solutions.

### Why multi-NT solutions rarely appear (not a bug)

dryadsynth-bv's exact-nullary engine is deterministic bottom-up,
size-stratified, smallest-tree-first, no randomization (`src/exact/
search.rs::run`). It returns the minimal-AST-size satisfying tree. For any
finite positive sample the globally smallest grammar is usually the trivial
"list the positives" enumeration `S0 -> w1 | w2 | ...`, which is 1-NT and
satisfies exhaustive negatives (it generates exactly the positives). So you
only see multi-NT when the sample *forces* it: long positives (listing
expensive) + an inherently >=2-NT language + exhaustive short negatives
(kill 1-NT recursive cheats via cross-contamination). The earlier
`gen_force_multint.py` failed this (short positives -> listing cheap).
`gen_force_union.py` ({a^n b^n} U {b^n a^n}, positives to length 6,
exhaustive negatives to length 5) is a genuine forcing instance: known
minimal solution is size ~23.

### The blowup is intrinsic — measured, don't re-attack with dedup tricks

`anbn_union_bnan_k2.sl` does not finish. Traced with per-size
instrumentation + `DELIS_DEDUP_STATS=1` (the `snapshot_line` path). Findings,
all measured:

- The search climbs sizes correctly toward the size-23 solution; candidate
  count grows ~3.6x per (odd) size. By size 17: ~230k candidates, ~28s.
  Direction is correct; it's just far.
- The blowup is entirely the `Rules` (production-list) NT. `Rhs` saturates
  (~603 distinct and flat); `Rules` explodes (879 -> 3851 -> 15783 ->
  61326 -> 229608 at sizes 9..17).
- Profiling (samply): ~41% self-time in `DefineFunPlan::eval` + ~25% in
  `Value` concat/clone/drop/malloc. That's a *symptom* of evaluating 230k
  candidates, not the cause. A 2x eval speedup buys ~1 size level (3.6x), so
  per-candidate throughput alone cannot close the ~5-level gap.
- **Not permutations/duplicates.** Theory-agnostic spine measurement
  (detect right-spine list NT by grammar shape; compare distinct ordered
  lists vs sorted multisets vs dedup sets of element structural hashes):
  `permutation=1.00x duplication=1.00x list/set=1.00x`. The 230k candidates
  are genuinely distinct production-sets. ACI / canonical-order
  normalization would buy nothing.
- **Not an over-fine upper sample.** The two-sample dedup key
  `(value@0, value@top)` shows `collapse@(0,top)=1.0x` and a growing gap to
  the single-sample floor (`distinct@0` vs `distinct@(0,top)` = 7.5x at
  size 17). Tightening the upper sample from all-ones down through F^2, F^4,
  to the **GFP** (tightest sound over-approximation) collapses only ~1.2x
  (size 13: 16360 -> 13699), nowhere near `distinct@0` (3655). The ~4.5x
  gap to the single-sample floor is *real recursive distinctness*; the
  single-sample key is unreachable because it's unsound, and here the
  unsoundness is load-bearing.

Conclusion: the `Rules` NT genuinely has ~3.6x/size distinct recursive
behaviors over these 64 models. No *sound* dedup refinement collapses it
(measured best case GFP = 1.2x). This benchmark sits near the genuine
difficulty frontier of its model-count / target-size regime, not behind a
fixable inefficiency. Do not re-attempt ACI canonicalization, upper-sample
tightening, or single-sample dedup on it. The only real levers are reducing
model count / aspect width (the keyspace multiplier) or using a smaller
forcing instance.

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

## TODO: PROSE encoding — re-evaluate Length-and-Char vs position-on-tape

The 354 PROSE files in `benchmark/String/` (translated by
`benchmark/String/scripts/gen.py`) currently use a **position-on-tape**
encoding: per-model `PosIn` and `PosOut` sorts, `Char` sort, char_at
lookups, and a dual-side semantics (`Str.SemIn` / `Str.SemOut`) where
the constraint pins `Str.SemOut` at the full output. `str.at` and
`str.substr` cross sides via disjunction; everything else is same-side.
`int.neg1`, `str.replace`, `int.indexof`, `str.int2str`, `int.to_int`
are stubbed `false`. Total disk: ~17 MB across 354 files; biggest is
2.4 MB (Address.000014 → 0014.sl). Most parse but solver wall time
is unverified for the larger files.

**Alternative under consideration — Length-and-Char (duet style).**
See `benchmark/misc/duet/11440431.sl` and its theory file. Each `StrProg`
gets two semantic projections — `StrProg.Len p asp → Idx` and
`StrProg.Char p (asp, k) → Chr` — instead of a `(lo, hi)` aspect.
`StrProg.OutEq` ties them via length-equality + pointwise-char equality
against the output. Two aspects: `ExAsp` (just model) and `CharAsp`
(model + k). No tape, no positions in the Str aspect itself.

Why this is interesting:

- Aspect space stays small per benchmark (no `O(out² )` substring
  enumeration; CharAsp scales with `max_len`, not `max_len²`).
- Closer in spirit to direct term-evaluation. Maps cleanly onto how a
  human thinks about FlashFill: "the synthesized program produces a
  string of some length, with character c at each position k".
- Krogmeier–Madhusudan–style aspect extension would just work if we
  ever need free-variable contexts.

What's not obvious:

- Lifting `str.substr s i n` into the (Length, Char) view without
  losing information. The duet theory file punts on the general case
  (returns `bot` for str.++ length, hardcodes specific patterns for
  suffixof). For 354 PROSE files we can't hand-author per-benchmark
  theory, so we'd need a uniform encoding.
- `str.replace` is the hard one — its length depends on the count of
  occurrences and the lengths of the replacement vs. the pattern.
- The Char projection at synthesized-position-k requires evaluating
  the synth-fun at every (model, k); the BV blowup may be
  comparable to the position-on-tape encoding.

Action items if/when we revisit:

1. Sketch the Length-and-Char encoding for one PROSE benchmark by
   hand. Compare BV bit count and file size against the current
   `0036.sl` (Column.000003) baseline.
2. Decide if we want a uniform encoding for all 354 or per-benchmark
   tailoring (duet does the latter; we'd need to autogenerate).
3. Krogmeier & Madhusudan, *POPL 2022*, "Learning Formulas in Finite
   Variable Logics" is the relevant paper. Local copy at
   `~/Zotero/storage/G9AFBVZX/`. Their encoding is parameterless
   (synth-fun is a constant formula); free variables enter via aspect
   extension on quantifiers, not as synth-fun parameters. We are
   *already* doing this for fixed-arity PROSE — `str.argi` are term
   constructors, not parameters — so the paper's contribution to
   PROSE specifically is about how to encode lambda-abstraction
   cleanly, not about synth-fun signatures.

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
