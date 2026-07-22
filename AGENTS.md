# Delis — Agent Guide

## Project overview

Delis is a Rust program-synthesis tool that learns logic formulas from finite
models. It compiles a Delis specification (SyGuS-IF extended with aspects and
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
| `dual.rs` | Automatic dualization (slated for removal in Phase 3) |
| `types.rs` | `AspectKey`, `AspectValue`, `FieldDomain`, `SemanticConstraint` |
| `tests.rs` | Compile-level unit tests |

## Active refactor: align with Facet++ paper

Approved plan: `/Users/wuchentian/.qoder/plans/eager-pool-pintail.md`

Three goals:
1. **Quantifiers range over bare sorts** — `(forall ((k Pos)) …)` instead of `(forall ((k (univ (Pos (model asp))))) …)`.
2. **Aspects are records of plain sorts** (no `(model Model)` field); model supplied at constraint level via `(with-model M …)`.
3. **Dual semantics declared manually** — remove `src/compile/dual.rs` auto-generation.

Guiding principle: keep `AspectKey.model` internally during migration, but
populate it from a compile-time `current_model` context rather than a
user-visible aspect field.

## Phase 1 — COMPLETE (all tests pass)

Bare sort symbols accepted as quantifier domains. Old `(univ …)` syntax still
works.

### Files changed
- `eval_ctx.rs`: `parse_univ_expr` accepts bare sort symbol → resolves via `current_model(env)`.
- `compiler.rs`: extracted `univ_values_for_expr` helper; `quantifier_values` accepts non-list domain exprs.
- `chc.rs`: same `quantifier_values` update.
- `bisim.rs`: `expand_range` delegates bare sorts to `parse_univ_expr`.
- `tests.rs`: added `bare_sort_quantifiers_use_current_model`.

## Phase 2 — NEARLY COMPLETE (one test assertion needs fixing)

Model-less aspects + `(with-model M …)` wrapper. Both old and new syntax
coexist.

### Files changed
- `eval_ctx.rs`:
  - `EnvView` trait gained `current_model()` default method (returns `None`).
  - `SemanticContext::current_model` prefers `env.current_model()`, falls back to aspect model.
  - `eval_aspect` rewritten: determines model from aspect field if present, else from `current_model(env)`. Arity error message changed from `"provides {} scalar fields, expected {}"` (with model-field subtraction) to `"provides {} fields, expected {}"` (no subtraction).
- `compiler.rs`:
  - `Env` struct gained `current_model: Option<String>`.
  - `constraint_model_env` sets `env.current_model` instead of injecting a fake aspect.
  - `resolve_constraint_model` replaces `extract_model_from_aspect`: checks if aspect space has Model field; if yes, extracts from args; if no, uses `env.current_model()`.
  - `lower_constraint` handles `(with-model M expr)` by cloning env and setting `current_model`.
- `chc.rs`: `Env` gained `current_model`; seeded from `bit.aspect.model`.
- `bisim.rs`: `WalkEnv` gained `current_model`; seeded from `aspect.model`.
- `aspects.rs`:
  - `peel_constraint_layers` strips `(not …)` and `(with-model M …)` wrappers, returns `(is_negative, outer_model, inner_term)`.
  - `parse_aspect_term` accepts `outer_model: Option<&str>`; synthetic `("model", AspectValue::Model(...))` binding added at the **beginning** of assignment (before field evaluation) so `:where` clauses and expressions like `(size (Pos model))` work for model-less aspects.
  - `enumerate_aspects` adds the same synthetic binding before `:where` evaluation.
  - `has_negative_semantic_constraint` and `collect_models_in_term` use `peel_constraint_layers`.
- `tests.rs`: added `model_less_aspects_use_with_model_context` (passes).

### Remaining fix for Phase 2

**`src/compile/tests.rs:999`** — update assertion from:
```rust
assert!(err.contains("provides 1 scalar fields, expected 2"));
```
to:
```rust
assert!(err.contains("provides 1 fields, expected 2"));
```

Then run `cargo test` to confirm.

### Caveat: leftover debug print

`src/compile/aspects.rs:632` has a stray `eprintln!("DEBUG parse_aspect_term assignment: {assignment:?}");` that must be removed.

### Caveat: pre-existing test failures (not caused by this refactor)

- `enumerates_subword_aspects_and_constraint_polarities` — assertion failure, was failing before changes.
- `parent_semantics_match_lfp_all_shapes` — missing external files/tools.
- `r0_support_edges_are_within_may_dep` — missing external files/tools.

### Caveat: synthetic "model" binding placement

The synthetic `("model", AspectValue::Model(model_name))` binding **must** be
pushed at the **beginning** of the assignment vector, before the field
evaluation loop. Field value expressions (e.g. `(size (Pos model))`) are
evaluated during the loop and need `model` already bound. Placing it after the
loop causes "expected Model expression, got Symbol(\"model\")" errors.

### Caveat: `expr.rs` finds model by scanning for `AspectValue::Model`

`eval_bool` and `eval_int` in `src/compile/expr.rs` locate the current model by
scanning the assignment env for an `AspectValue::Model` entry. The synthetic
binding makes this work for model-less aspects. No changes were needed in
`expr.rs` itself.

## Phase 3 — NOT STARTED

- Delete `src/compile/dual.rs`.
- Remove `dualize_negatives` logic and `dual::dual_specs` call from `aspects.rs`.
- Remove `dualize_negatives` parameter from `collect_semantic_constraints` and related helpers.
- CHC paired positive/negative predicate emission stays (it's part of the CHC encoding, not auto-dual).
- Add explicit error: if program is parent-bearing and has negative semantic constraints without a user-declared dual relation, emit a clear error explaining a manual dual evaluator is required.
- `src/compile/mod.rs:14` still declares `mod dual;` — remove when module is deleted.

## Benchmark migration — NOT STARTED

Mechanical syntax changes across `benchmark/**/theories/*.sl`:
- Replace `(forall ((k (univ (Pos (model asp))))) …)` with `(forall ((k Pos)) …)`.
- Remove `(model Model)` from aspect records.
- Wrap constraints in `(with-model M …)`.

Do family by family, running tests after each.

## Build & test

```sh
cargo test                    # run all tests
cargo test -- --skip parent   # skip parent-bearing tests needing external tools
cargo run -- --dump-sygus <file.sl>  # inspect generated SyGuS
```

Release profile uses thin LTO, codegen-units=1, panic=abort.
