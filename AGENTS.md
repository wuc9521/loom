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

### Vendored sygus-parser — supports zero-argument applications

The `sygus-parser` crate is vendored at `sygus-parser/` and patched via `[patch.crates-io]` in `Cargo.toml`.

Patches:
- Grammar (`sygus.v2.pest`): `SyGuSTerm+` → `SyGuSTerm*` (and `SyGuSBfTerm+` → `SyGuSBfTerm*`) to allow zero-argument applications like `(FO.Asp)`.
- AST construction (`ast/terms.rs`): When a `SyGuSTerm` has a single `Identifier` inner pair but the matched string starts with `(`, construct `Application(identifier, vec![])` instead of `Identifier`. This distinguishes `(FO.Asp)` from bare `FO.Asp`.

The local Loom grammar (`src/parser/sygus.v2.pest` and `src/parser/loom.pest`) has the same `*` fix for consistency, though the actual constraint parsing goes through the vendored crate.

### Benchmark migration — COMPLETE

All `.sl` files under `benchmark/` migrated to paper-aligned syntax. Verified: 0 files with `(model Model)`, 0 files with `(univ …)`.

### Pre-existing test failures (not caused by this refactor)

- `enumerates_subword_aspects_and_constraint_polarities` — assertion failure.
- `parent_semantics_match_lfp_all_shapes` — missing external files/tools.
- `r0_support_edges_are_within_may_dep` — missing external files/tools.

## Performance invariants — profile-driven, do NOT revert

CTL compile hot paths run 10M+ times per instance (measured with macOS `sample`;
bisim phase was 98.5% of worst-case runtime). Rules:

- `eval_ctx.rs`: `eval_cache` is keyed by a `u64` FxHash of (model cache_id,
  func, args) — allocation-free probe. Do not restore the old String-based
  `EvalKey` (allocated ~3 Strings per probe).
- Hot paths use `simple_sort_name()` (Cow) / `Sort::as_simple_name()`; Bool
  checks compare `as_simple_name() != Some("Bool")`, never `to_string()`.
- `compiler.rs`: `guarded_projection_bits` returns `Option` and doubles as the
  constant-guard probe — a memo hit must NOT re-enumerate the domain
  (`const_guard_values` was ~69% of the encode phase).

Any change here must keep `--dump-sygus` output byte-identical on
`benchmark/CTL/0002.sl` (default) and `0542.sl --bisim=false`.

## Build & test

```sh
cargo test                    # run all tests (43 pass, 3 pre-existing failures)
cargo test -- --skip parent   # skip parent-bearing tests needing external tools
cargo run -- --dump-sygus <file.sl>  # inspect generated SyGuS
```

Release profile uses thin LTO, codegen-units=1, panic=abort.
