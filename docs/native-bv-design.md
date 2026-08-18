# Native-BV emission for numeric finite universes

## Problem

loom compiles finite-model semantics to SyGuS-BV generically: universe
elements become one-hot bits, and every operation over them expands over
universe elements at emission time. For domains whose universes are secretly
numbers (String: positions, lengths, char codes), this is the dominant cost:

| construct | today's emission | cost |
|---|---|---|
| `(le.pos X Y)`, both args semantic calls | OR over all table-true pairs `(a,b)`, each an AND of two bit tests | O(\|U\|²) |
| `(= Z (plus.pos X Y))` | per target value, OR over arg tuples | O(\|U\|²)–O(\|U\|³) |
| `(= X Y)` between calls | OR over values of paired bit tests | O(\|U\|) |

Measured consequences (String suite): multi-MB `str.substr`/`str.replace`
define-funs, 30–150 MB SyGuS outputs, compile-time budget gates
(`char_ops`/`charis`/`replace`) that exclude large-L and many-model files.

By contrast, an interpreter-style solver (ks2 on the semgus value-vector
encoding) evaluates the same semantics over native ints in microseconds.
dryadsynth-bv *executes* define-funs on concrete bitvectors during
enumeration, so if define-funs were compact native-BV programs, execution
would be interpretation — while keeping loom's single generic pipeline and
its abstraction lever (demand-style states) that interpreters cannot have.

## Key observations

1. **The one-hot value field already supports native ops.** A FiniteValue
   relation instance (e.g. `Str.Len x` at one aspect) occupies one bit per
   universe value; gathering those bits (in listed-universe order) into a
   word `f` gives a one-hot encoding `f = 2^v`. Then, writing `U` for the
   listed order:
   - order: `2^a ≤u 2^b ⟺ a ≤ b` → `bvule` works directly on one-hot;
   - equality: `bvand(fx, fy) ≠ 0` (also correct when a side is zero-hot);
   - addition: `2^a · 2^b = 2^(a+b)` → `bvmul` on zero-extended fields
     compares saturating sums without materializing them.

2. **No language change is needed: recognize tables instead of annotating.**
   The match tables still carry the full semantics, and they are tiny.
   At compile time, for each 2-ary model function, check exactly whether the
   table equals `≤` / `<` / saturating `+` / saturating `−` over the listed
   universe order, with an optional trailing *invalid* element ($inv/$neg)
   that behaves as "no row" (Bool tables) or as the saturation target
   (value tables). Recognition is exact and per-model; anything that does
   not match falls back to the existing expansion. This also benefits any
   future suite whose tables happen to be numeric, with zero opt-in.

3. **bisim interacts with fields.** The quotient may remap bit indices, so
   fields are built by explicit `concat` of the per-value bit terms (via
   `semantic_term_bit`, which already handles child references and
   SelfClosure placeholders). If the quotient merged two values of the same
   instance onto one bit, the one-hot reading would be wrong — the builder
   checks index distinctness and falls back.

## Tier 1 (this change): template emission at the expansion sites

- `src/compile/native.rs`:
  - `TableKind = { Ule, Ult, UleInvBig, AddSat, SubSat }`
    (`UleInvBig` covers `ltl.pos`-style "invalid RHS means +∞").
  - `recognize(model, func) -> Option<TableKind>`, cached per (model, func).
  - `value_field(expr, env) -> Option<Field>`: one-hot field for a
    FiniteValue semantic call / universe constant / bound value.
- Hooks:
  - `eval_model_bool_call`: recognized 2-ary Bool table + extractable args →
    emit comparison template.
  - `eval_finite_eq` / `eval_finite_terms_equal`: argument that is an
    application of a recognized AddSat/SubSat function → emit arithmetic
    template on zero-extended fields.
- Every template guards the invalid element and zero-hot fields explicitly
  so it is point-wise equal to the table expansion (differential-testable).

## Tier 2 (future): packed binary state

Replace one-hot value fields with ceil(log2 |U|)-bit packed fields, mapping
recognized ops to `bvadd/bvsub/bvule` directly and shrinking state width.
Requires bit-layout, constraint-lowering, and bisim group awareness; only
worth it after Tier 1's evaluation-cost win is banked.

## Expected effect

The compile budgets in the String generator exist only because of the
expansions Tier 1 removes; with them gone, `char_ops`/`charis`/`replace`
can widen toward unconditional, bringing the currently excluded large-L and
many-model files (0161, 0309, 0334, 0337, 0332, …) into range.

## Implementation notes (Tier 1 landed)

- **Pitfall: dryadsynth-bv's boolean algebra is active-low and 1-bit-only.**
  SMT `=` parses to a bitwise xor, `or` to a bitwise and, `not` to bitnot —
  correct only for 1-bit carriers. Word-level equalities/nonzero-tests from
  these templates must therefore be phrased with normalizing producers:
  `1 <=u x` for `x != 0`, `ule` both ways for word equality (see
  the abandoned word-op emitter). The final gate-level emitter produces only
  1-bit carriers, so no dryadsynth-bv change is required (a width-safety
  hardening of `eval_op_on_values` was prototyped and then reverted; worth
  upstreaming separately if multi-bit booleans ever reach that path).
  Kill switch for differential runs: `LOOM_NO_TABLE_CIRCUITS=1`.
- **Verification**: template-on/off solutions identical on all probes and
  0031/0205; z3 equivalence check of the emitted `str.at` (one-hot-constrained)
  is unsat (`equiv2.smt2` method — extract both define-funs, assert disequal).
- **Measured, compile** (`--sygus-stats`): 0205 39.8s -> 10.1s,
  0230 2.9s -> 2.1s, 0031 0.56s -> 0.47s. Output bytes grow ~1.5x because
  each template use re-emits its field concat (no sharing).
- **Word-op templates were a NET LOSS end-to-end** (0031 solve 2.4s -> 31s):
  dryadsynth-bv's fast plan is a *bitsliced* BitCircuit evaluator (64
  candidates per machine word over a shared boolean DAG), and `bvmul`/`bvule`
  knocked the define-funs down to the Generic per-candidate interpreter.
  The huge table expansions were never paid per candidate — they cost
  ~gates/64 each.
- **Resolution: emit circuit gadgets, not word ops.** The recognized ops are
  lowered to pure 1-bit and/or/not with shared suffix-OR chains (`a <= b` on
  one-hot = OR_i(a_i AND suffixOR_i(b)), O(n) gates vs O(n^2) unshared
  pairs; +/- against constants fold into index shifts). Everything stays on
  the BitCircuit plan; sides are restricted to one dynamic field + constant
  offset, which covers every pattern the String generator emits.
- **Measured, gate-level, default-on with -O**: compile 0205 38.1s -> 9.1s,
  0230 2.7 -> 1.8s; solve 0021 68 -> 38s, 0190 45 -> 40s, 0031/0144/0224
  parity; solutions identical everywhere; Reg/LTL/CTL/Reg+ samples byte-for-
  byte unchanged (no numeric tables there, recognition never fires).
- True machine-word arithmetic (bvadd on packed fields) only pays off with a
  packed-value *batch* evaluator — that is Tier 2's territory.

## Next steps

1. Widen the String generator budgets (char_ops/charis/replace) now that the
   dominant expansions are O(n): re-tune on the bench machine.
2. Sides with two dynamic fields (e.g. `plus` of two calls in comparisons)
   currently fall back to expansion; distribute over the smaller field if
   profiling shows them hot.
3. Tier 2 (packed fields + packed-value batch evaluator) for true native
   arithmetic.
