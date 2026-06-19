# delis — current state (handoff)

This file is the authoritative working notes. Older revisions of this file
were stale; trust THIS file over anything you remember about prior contents.

## What delis does (one-liner)

Compiles a typed semantic DSL (`define-semantics` over `define-aspects` /
`declare-term-types`) into SyGuS-BV and dispatches to `dryadsynth-bv` (or
an external SyGuS solver) to synthesize a term satisfying labeled examples.
Two input shapes:

- **No `(parent n)`**: per-bit BV encoding is sound + complete (verified
  via `tools/smt-crosscheck.sh` and end-to-end solve on Reg/0001).
- **With `(parent n)`** (CFG, FO with up-moves, …): per-bit BV alone is too
  weak — see the validated approach below.

## Pipeline (post-saturation Comon §7.6.3 style)

```
DelisProg
  → SemanticProgram (bisim quotient at aspect level — src/compile/bisim.rs)
  → HornProg via compile_chc (src/compile/chc.rs)
  → push×pop saturation (src/compile/saturate.rs)
  → BitLayout (src/compile/layout.rs — one bit per HornProg.predicates entry)
  → aggregate_constructor_bits (src/compile/emit.rs)
  → per-cons define-funs + per-example bit-extract constraints (sygus_parser)
  → kleene::apply (src/compile/kleene.rs) — wraps parent-referencing
    nullaries in a synth-fun-with-args + Kleene unrolling chain when the
    HornProg has POP rules.
```

## Validated approach: Kleene unrolling for parent specs

Experimentally validated answer to the open question that killed earlier
iterations of this file (Sessions C through O). It composes:

1. **Encoding (steps 1-3 of K&M Lemma 1 chain)**: the Horn IR is sound +
   complete for the Horn LFP semantics. Saturation is Comon §7.6.3 push×pop
   resolution (no case-split — case-split was experimentally unsound at
   per-bit BV, verified by 3-solver crosscheck on CFG/0001).
2. **Bit-derivation (the auxiliary fix)**: parent-referencing nullaries
   (constructors with 0 args whose sort is used as an arg) get bit
   patterns that DEPEND on g. Per-bit BV can't bake this in statically;
   we add a Kleene chain `c_0 = baseline; c_{i+1} = c_baseline ∨ (extract
   tgt_bit (g <c_*_i>))` at SMT solve time, for `N` iterations.
3. **Synth-fun gets args**: each parent-referencing nullary becomes a
   synth-fun argument. The grammar's terminal references to those nullaries
   get rewritten to the arg vars. Constraints apply `(g a b c)` where
   `a, b, c` are the user-visible aliases for `c_*_N`.

### Verified outcome

3-solver crosscheck (z3 / bitwuzla / cvc5) on CFG/0001 candidates at `N=2`:

| candidate (encodes) | expected | got |
|---|---|---|
| `S → ab | aSb` (recursive) | sat | sat ✓ |
| `S → ab | aaSb` (unsound) | unsat | unsat ✓ |
| `cfg.top.S0 cfg.rhs.S0 end` (degenerate) | unsat | unsat ✓ |
| `S → ab | aabb | aaabbb` (enumeration) | sat | sat ✓ |

Generalizes to K=2 (verified on tiny `aaa` cases in
`tools/tiny_cfg_comon.py` — same N-step approach works at multiple K
with zero per-K code).

### CLI

`delis --kleene N <spec>` sets the unrolling depth. `0` disables (no-op
for parent-free specs anyway, since `bit_link::derive_bit_links` returns
empty without POP rules in the pre-saturation HornProg).

## Status of the end-to-end synth (the still-open piece)

The SyGuS-2.1 file delis emits for parent specs (`g` with args + Kleene
chain) is structurally correct and verified by external SMT solvers.
**dryadsynth-bv's existing search paths don't yet handle it** — its
`OwnedExpr::from_term` errors with "Unrecognized Expression" when it
encounters `(g a_0 b_0 cfg_rhs_S0_0)` (a synth-fun in Application position).

Two shapes dryadsynth-bv DOES handle natively:
- Synth-fun without args (bit-constraint or PBE-literal-tuple).
- Synth-fun with args under PBE shape `(= (g lit lit lit) lit)` only —
  via `parse_pbe_from_view`, requires args to be `SyGuSTerm::Literal`.

Our shape `(= ((_ extract bit) (g define_fun_a define_fun_b define_fun_c)) #b1)`
fits neither (args are define-funs, not literals).

The proper extension in dryadsynth-bv (in progress):
- Add `OwnedExpr::SynthApp(usize, Vec<OwnedExpr>)` variant.
- Recognize synth-fun symbols in `from_term`'s Application branch.
- Evaluator substitutes the candidate's body at each SynthApp during search.
- Generalized constraint parser puts non-PBE-shaped constraints into
  `exact::Constraint.structural` (already a `Vec<OwnedExpr>` — the IR
  exists; only the parser needs to fall through).
- Backward relevance walks SynthApp's args and the synth-fun's grammar
  for demanded output bits.

## Key files

- `src/compile/kleene.rs` — generic Kleene unrolling: detects
  parent-referencing nullaries, rewrites synth-fun signature, emits
  baselines + iteration chain. 3 unit tests pass.
- `src/compile/bit_link.rs` — auto-derives `(child_cons, src_pred, tgt_pred)`
  triples from POP rules (must be called on PRE-saturation HornProg —
  saturation consumes pops).
- `src/compile/emit.rs::compile_via_horn` — orchestrates the pipeline.
  Extracts bit_link triples before saturation, calls kleene::apply at the
  end with `opts().kleene_n`.
- `src/compile/saturate.rs` — push×pop resolution + alpha-subsumption.
  Case-split intentionally NOT done (experimentally unsound for delis at
  per-bit BV; the SOUND fix is Kleene unrolling outside saturation).
- `tools/tiny_cfg_comon.py` — standalone Python prototype that validated
  the 4-step pipeline + Kleene on K=1 and K=2 tiny CFG cases. 10/10
  candidates verify across solvers. Reference implementation.
- `tools/lfp-emit.py` — older Python verifier (works on a given candidate
  via text substitution). Useful for one-off crosschecks.
- `tools/smt-crosscheck.sh` — ground truth: substitute a candidate into
  the emitted SyGuS and check 3-solver agreement.

## Things that DID NOT work (don't relitigate without strong reason)

- **alt_enum**: hand-rolled "OR LFP-reachable bits onto parent
  nullaries". Sound for positives, **unsound** for negatives (admits
  degenerate `(cfg.top.S0 cfg.rhs.S0 end)`). Kept in tree as a reference
  for the demand-walk shape; `emit.rs` does NOT call it. Verified
  unsoundness by 3-solver crosscheck.
- **Comon case-split inside saturate**: produces body-less facts that
  are sound in Horn LFP semantics but unsound for delis (over-approximates
  parent context). Verified by experiment (zeroing cfg.rhs.S0 flips the
  unsound candidate's verdict, isolating the unsoundness to the spuriously
  derived bits). The functions `case_split` and `try_case_split` have
  been removed from saturate.rs.
- **subset_dta**: subset construction at the per-bit level. Algorithm
  correct on small hand examples (4 unit tests in `subset_dta.rs`).
  Doesn't scale at delis predicate granularity (Reg/0001 hits 145
  subsets after one round; cartesian over child subsets blows up).
  Kept in tree for the algorithm but not wired in.
- **fwd-decls in dryadsynth-bv**: experimental multi-synth-fun engine.
  Removed in commit `cfe3bdc` (in sibling crate).

## Soundness verification workflow

Always: emit → crosscheck → measure. Don't trust analysis without
empirical confirmation. `tools/smt-crosscheck.sh <spec> <candidate>` is
the ground truth. Three-solver agreement (z3 + bitwuzla + cvc5)
distinguishes encoding-level verdicts from solver quirks.

If any new encoding work changes how parent-referencing nullaries are
emitted, the 4 CFG/0001 candidates from the table above are the minimum
gate. For K=2 work, add 4 analogous candidates per
`tools/tiny_cfg_comon.py`.

## Reproducing

```bash
cargo build --release

# Parent-free spec — solves immediately.
./target/release/delis benchmark/Reg/0001.sl

# Parent spec — emits the correct SyGuS file (verifiable by external
# solver) but currently fails in dryadsynth-bv pending the SynthApp
# extension. Dump and crosscheck manually:
./target/release/delis --kleene 2 --dump-sygus benchmark/CFG/0001.sl > /tmp/cfg.sl
# substitute candidate into /tmp/cfg.sl per tools/smt-crosscheck.sh shape.

# Validate via Python prototype (works today):
python3 tools/tiny_cfg_comon.py
```

## Coding preferences (Chentian)

- Function names short. Modules don't leak concerns from elsewhere
  (`compile/kleene` knows about HornProg + signature, not about specific
  spec families).
- Algorithm input-agnostic: depends only on what `delis.pest` admits
  (`define-aspects` / `define-semantics` / `declare-term-types` /
  `define-model` / `declare-fun`). NO hardcoded CFG/LTL/Reg knowledge.
- Real algorithmic wins, not Rust micro-tweaks.
- "Think harder" = look at data. Profile/measure first.
- **Never add `#[allow(dead_code)]`** — remove the dead code or scope
  to `#[cfg(test)]`.
- Don't commit dead/unused code, dead-code annotations, or
  backwards-compat shims.

## What to do next (handoff TODO)

### SynthApp extension in dryadsynth-bv — **PARTIAL, in progress**

Done so far in `../dryadsynth-bv` (uncommitted on this branch):
- `OwnedExpr::SynthApp(usize, Vec<OwnedExpr>)` variant + Display helper
  (`src/enumerate/macros.rs`, `src/enumerate/expr.rs::format_owned_synth_app`).
- Thread-local synth-fun registry in `src/parse/deffun.rs`
  (`set_synth_registry` / `lookup_synth`).
- `OwnedExpr::from_term` recognises synth-fun symbols and produces
  `SynthApp` (`src/enumerate/expr.rs`).
- `solve_file_inner` populates the registry before parsing
  (`src/lib.rs:94-99`).

**Items 1-3 + 5 done; current blocker is item 4 (the dispatcher).**
Running `delis --kleene 2 benchmark/CFG/0001.sl` now reaches the
dispatcher in `lib.rs::solve_file_inner` and errors with:
```
"Unsupported SyGuS benchmark shape for g: expected PBE examples,
 a define-fun reference implementation, or a nullary bit-constraint task"
```

What landed (uncommitted on `../dryadsynth-bv`):
1. **`SynthEntry { name, arity, ret_bits }`** registry; `synth_ret_bits`
   / `synth_arity` helpers. `width()` (`expr.rs`) and `bit_width()`
   (`algo.rs`) use these — no more `unimplemented!()`.
2. **`PlanInstr::SynthCall { synth_idx, args }`** variant +
   `compile_expr` case + plan executor case. Plan-eval reads the
   thread-local candidate, recurses into its body via
   `eval_with_bindings`.
3. **Thread-local `CURRENT_CANDIDATES: RefCell<Vec<Option<OwnedExpr>>>`**
   with `set_synth_candidate(idx, body)` / `get_synth_candidate(idx)`.
   `OwnedExpr::eval_with_bindings` for `SynthApp` substitutes.
5. **Backward relevance** through `SynthCall`: conservative
   `BitDep::full(arg_bits)` (single-bit walker) and "demand all bits of
   all args" (forward-demand walker). Sound but loose. Tightening is an
   optimisation; not blocking.

Remaining:

4. **Dispatcher + constraint parser** — the real open work. Our shape:
   ```
   (constraint (= ((_ extract N N) (g a b cfg.rhs.S0)) #b1))
   ```
   has arity=3 but the args are nullary define-fun calls (the Kleene
   chain), not literals. `parse_extract_call_constraint` requires
   literal args, so `parse_bit_constraints_grouped` returns `None`, and
   the dispatcher cascades through PBE / reference-impl / nullary-bit
   paths and ends at "Unsupported".

   Proposed path: in `parse_bit_constraints_grouped` (arity > 0 branch),
   if `parse_extract_call_constraint` returns None but
   `constraint_mentions_target` is true, lift the whole constraint via
   `OwnedExpr::from_term` and push to `structural`. The lifted expr
   contains `SynthApp(0, [Func(a), Func(b), Func(cfg.rhs.S0)])`. The
   search machinery's `spec.matches(&values)` then needs to set the
   current candidate before evaluating structural exprs — which means
   threading the candidate's `OwnedExpr` (not just its `values`) into
   the search loop. `try_push_new` / `try_push_existing` only have
   `Node`; would need a `materialize → set_synth_candidate(0, expr)`
   pair before `matches`.

   This is the big remaining chunk. Once `set_synth_candidate` is
   called at each candidate, the structural eval already works
   (`SynthApp` thread-local read is in place).

### Other follow-ups

6. **Run end-to-end synth** on CFG/0001, CFG/0002, CFG/0003 once the
   above lands. Crosscheck against the Python prototype.
7. **K=2 specs** (`benchmark/CFG/0026.sl` etc.) need larger Kleene N
   (empirically N=3 for tiny `aaa`; conservatively `N = max_word_len`).
8. **delis orchestrate loop** `N := 1; N += 1` in `src/main.rs::solve`
   — wrap the dryadsynth-bv call, increment on UNSAT. Small (~30 LOC).
