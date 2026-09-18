# Loom — Agent Guide

## Project overview

Loom is a Rust program-synthesis tool that learns logic formulas from finite
models. It compiles a Loom specification (SyGuS-IF extended with aspects and
semantic relations) into either a SyGuS bit-vector encoding (`to_sygus`) or a
CHC encoding (`to_chc_ast` for parent-bearing programs).

Key source modules live under `src/compile/`:

| Module | Responsibility |
|---|---|
| `aspects.rs` | Aspect space definition, enumeration, constraint parsing |
| `compiler.rs` | Direct BV/SyGuS compilation path (parent-free) |
| `chc.rs` | CHC compilation path (parent-bearing) |
| `eval_ctx.rs` | `SemanticContext`, `EnvView` trait, `eval_aspect`, `parse_univ_expr` |
| `expr.rs` | `eval_bool`, `eval_int` over aspect assignment envs |
| `bisim.rs` | Bisimulation quotient for state-space reduction |
| `types.rs` | `AspectKey`, `AspectValue`, `FieldDomain`, `SemanticConstraint` |
| `tests.rs` | Compile-level unit tests |

## Facet++ paper alignment — ALL PHASES COMPLETE

Approved plan: `/Users/wuchentian/.qoder/plans/eager-pool-pintail.md`

Three goals:
1. **Quantifiers range over bare sorts** — `(forall ((k Pos)) …)`.
2. **Aspects are records of plain sorts** (no `(model Model)` field); model supplied at constraint level via `(with-model M …)`.
3. **Dual semantics declared manually** — no auto-generation.

### Phase 1: Quantifiers over sorts — COMPLETE

`parse_univ_expr` accepts bare sort symbols, resolves model via `current_model(env)`.

### Phase 2: Model-less aspects + `(with-model M …)` — COMPLETE

- `EnvView` trait has `current_model()` method.
- `eval_aspect` determines model from aspect field if present, else from `current_model(env)`.
- `lower_constraint` handles `(with-model M expr)` by setting `env.current_model`.
- `parse_aspect_term` accepts `outer_model: Option<&str>` for model-less aspects.
- Synthetic `("model", AspectValue::Model(...))` binding pushed at beginning of assignment vector.

### Phase 3: Remove automatic dualization — COMPLETE

- Deleted `src/compile/dual.rs`.
- Parent-bearing programs with negative semantic constraints emit a clear error requesting a manual dual.

### `(univ ...)` syntax eliminated — COMPLETE

The `(univ (Sort (model asp)))` syntax has been fully removed from the codebase.

Changes:
- `eval_ctx.rs`: `parse_univ_expr` only accepts bare sort symbols; the `univ` branch is removed.
- `bisim.rs`: Removed the `"univ"` case from `expand_range`.
- `tests.rs`: Updated tests to use bare sort names.
- FO benchmark `.sl` files: Migrated `(univ (Sort))` → bare `Sort`.
- Python generator scripts (`fol2sl.py`, `gen_eqstyle.py`, `gen_eqstyle_v1.py`): Updated to emit bare sort names.

### `(get-fun F M args)` syntax eliminated — COMPLETE

The explicit-model accessor was redundant with `(with-model M (F args))` —
every String benchmark already wrapped its get-funs in the matching
`with-model`. All 149,410 occurrences across `benchmark/String/*.sl` were
mechanically rewritten to bare model calls (script validated each rewrite
against the enclosing `with-model`), and the `get-fun` branch plus its
`ConstraintModelEnv` were deleted from `eval_ctx.rs`. Verified byte-identical
`--dump-sygus` output on String samples (0001-0004, 0030, 0036, 0171).

### Vendored sygus-parser — supports zero-argument applications

The `sygus-parser` crate is vendored at `sygus-parser/` and patched via `[patch.crates-io]` in `Cargo.toml`.

Patches:
- Grammar (`sygus.v2.pest`): `SyGuSTerm+` → `SyGuSTerm*` (and `SyGuSBfTerm+` → `SyGuSBfTerm*`) to allow zero-argument applications like `(FO.Asp)`.
- AST construction (`ast/terms.rs`): When a `SyGuSTerm` has a single `Identifier` inner pair but the matched string starts with `(`, construct `Application(identifier, vec![])` instead of `Identifier`. This distinguishes `(FO.Asp)` from bare `FO.Asp`.

The local Loom grammar (`src/parser/sygus.v2.pest` and `src/parser/loom.pest`) has the same `*` fix for consistency, though the actual constraint parsing goes through the vendored crate.

### Benchmark migration — COMPLETE

All `.sl` files under `benchmark/` migrated to paper-aligned syntax. Verified: 0 files with `(model Model)`, 0 files with `(univ …)`.

## Performance invariants — profile-driven, do NOT revert

CTL compile hot paths run 10M+ times per instance (measured with macOS `sample`;
bisim phase was 98.5% of worst-case runtime). Rules:

- `eval_ctx.rs`: `eval_cache` is keyed by a `u64` FxHash of (model cache_id,
  func, args) — allocation-free probe. Do not restore the old String-based
  `EvalKey` (allocated ~3 Strings per probe).
- Hot paths use `simple_sort_name()` (Cow) / `Sort::as_simple_name()`; Bool
  checks compare `as_simple_name() != Some("Bool")`, never `to_string()`.
- Hot paths read list operands with `as_slice()`, never `to_ref_vec()`
  (allocates a `Vec<&Value>` per call): `eval_bool_value`, `eval_value`,
  `eval_int_expr`, `bisim::walk`.
- `eval_ctx.rs`: `eval_bool_value` dispatches builtin ops on the first byte
  of `op` before string-compares (~25M calls per quotient; the literal match
  ran up to 7 full string compares per call). Guard failure falls through to
  the default arm exactly as the plain match did — keep that equivalence.
- `bisim.rs`: `signature()` takes caller-owned `WalkEnv`/`WalkCache` scratch
  and `clear()`s them per call — do not restore per-call construction.
- `bisim.rs`: keep the `WalkCache` scoped per `signature()` call. A persistent
  walk cache (per-round or per-quotient; soundness via recorded deps +
  per-class split generations) was built and measured SLOWER — 0002 bisim
  2.68s -> 3.11s — because the shared table grows to ~10M entries and every
  probe becomes a DRAM random access, while per-call tables stay
  cache-resident. Do not retry without fixing locality first.
- `bisim.rs`: parallel refinement was built and measured SLOWER — do not
  retry as-is. The load-bearing fact is deterministic: batched (Jacobi)
  snapshot rounds triple total work (63 iters / 38.2M evals vs 22 / 12.7M —
  counters, not timings; the within-round Gauss-Seidel propagation is worth
  3×), so a parallel variant must sustain >3× thread scaling just to tie
  the sequential fixpoint. Measured scaling came nowhere near (2 threads:
  1.11×, 4: 1.34×, 8: 1.06×, 10: 0.94× total; at 10 workers each signed
  1/10 of a round's pairs in the same wall time one worker signs all) —
  but those runs shared the machine with another CPU hog, so treat the
  coefficients as indicative only. Suspected cause: each worker's private
  working set (per-call walk cache, eval caches, env maps) duplicates what
  fits in shared L3 when single-threaded, so N workers thrash it N×.
  Output was verified byte-identical at every thread count — the fixpoint
  argument holds; only the performance died.
- `compiler.rs`: `eval_model_bool_call`'s ground probe delegates to
  `ctx.eval_model_bool_call` (u64 key over arg VALUES, allocation-free
  probe) — do not restore the per-call `ground_arg_names` path there: it
  rebuilt display Strings + per-arg `EnvId` fingerprints for every call
  (~25% of the encode phase). `Err` = args not ground = fall through to
  dynamic encoding, same as the old `None`. Value-keyed means equal
  (func, args) calls hit across envs, unlike the env-keyed `eval_bool`
  memo. `eval_model_finite_call_eq` still uses `ground_arg_names` (its
  target match is `interped_value_matches`, not `values_equal` — do not
  delegate it without checking that equivalence).
- `compiler.rs`: `guarded_projection_bits` returns `Option` and doubles as the
  constant-guard probe — a memo hit must NOT re-enumerate the domain
  (`const_guard_values` was ~69% of the encode phase).
- `bisim.rs`: `expand_range` returns the `Arc<Vec<AspectValue>>` from
  `SemanticContext::univ_cache`, memoized per (expr ptr, model) in
  `range_cache` — the universe is static; rebuilding it per expansion cost
  ~25M allocations per quotient. `emit_quantifier` binds
  its env key once and overwrites the slot per value (no per-value String
  clone, no per-value save/restore).
- `eval_ctx.rs`: `univ_cache` memoizes each (model, sort) universe as
  `Arc<Vec<AspectValue>>`. `eval_value_for_sort` and `argument_univs` must
  hit it, not rebuild via `univ_for_sort` (owned `Vec<String>` clone = N+1
  allocs per call; the cache cut 0002 bisim 2.18s -> 1.77s). Universes are
  static for the run — same invariant as `lang/eval.rs`'s `UNIV_CACHE`.
- `eval_ctx.rs`: `eval_value_ref` is the borrowed fast path for env-resident
  values (bare bound symbol, `(field asp)` field) — same branches, same
  precedence as `eval_value`, `None` = fall back to owned. The `=` arm of
  `eval_bool_value`, `with_value_for_sort` (so `model_call_key`'s per-arg
  hashing on the 12.7M-call path), and bisim's `try_emit_value` are
  ref-first; do not restore owned `eval_value` clones there. RefCell-backed
  caches can never join this pattern (refs across recursive evals risk
  borrow panics); `(model asp)` is excluded (owned String clone).
- `eval_ctx.rs`: `eval_model_bool_call` / `eval_model_value_call` take the
  caller-resolved `&FuncSig` — do not re-fetch `func_sigs` inside (two
  string hash+memcmp per eval call). `sig.name` bytes == the map key, so
  key hashes are unchanged.
- `compiler.rs`: `EnvId` is a u64 FxHash fingerprint (built by
  `memo_env_key`), not structure. Extra discriminators (relation, sort,
  finite-eq target) fold into the same u64 via `EnvId::mixing`, `bit_index`
  is keyed by the `BitKey::fingerprint` u64 of (relation, aspect, value),
  and `memo_env_key` hashes symbol names by cache-entry pointer, not bytes —
  no probe allocates a key field. Two soundness rules, both violations of
  which were caught only by byte-diffing output:
  1. Hash the FULL `AspectKey` of a referenced binder — never restrict to
     syntactically-referenced field names. Bodies pass `asp` wholesale to
     Sem/Asp constructors (`(CTL.Sem phi asp)`), which read fields no
     projection ever names.
  2. `GuardedProjectionMemoKey` carries env views of all three exprs
     (domain, guard, aspect): the cached bits depend on each.

Any change here must keep `--dump-sygus` output byte-identical on
`benchmark/CTL/0002.sl` (default) and `0542.sl --bisim=false`, plus a sweep
over `benchmark/CTL/*.sl` against the previous binary.

## Build & test

```sh
cargo test                    # run all tests (45 pass, 1 ignored)
cargo test -- --skip parent   # skip parent-bearing tests needing external tools
cargo run -- --dump-sygus <file.sl>  # inspect generated SyGuS
```

Release profile uses thin LTO, codegen-units=1, panic=abort.
