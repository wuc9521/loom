# Loom

Loom learns logic formulas from finite models. It lowers a Loom specification
to SyGuS bit-vectors, races its internal enumerator against the in-process
[`loom-bv`](loom-bv) solver, and prints the first result.

## Build

Requires nightly Rust ([`rust-toolchain.toml`](rust-toolchain.toml)).
`loom-bv` and `benchmark/` are git submodules;
[`sygus-parser`](https://github.com/wuc9521/sygus-parser-rs) comes from crates.io.

```bash
git clone --recursive https://github.com/wuc9521/loom.git
cd loom
cargo install --path . --locked
```

In an existing clone, run `git submodule update --init --recursive` first.

## Example

[`benchmark/Reg/0003.sl`](benchmark/Reg/0003.sl) asks for a regex that matches
every word containing `0101`, given positive and negative example words:

```lisp
(synth-fun r () RegLan)

(constraint (with-model w0101 (RegLan.Sem r (RegLan.Asp head tail))))
(constraint (not (with-model w0 (RegLan.Sem r (RegLan.Asp head tail)))))
...
```

```bash
$ loom benchmark/Reg/0003.sl
(
(define-fun r () RegLan (re.++ (re.* re.allchar) (re.++ char.0 (re.++ char.1 (re.++ char.0 (re.++ char.1 (re.* re.allchar)))))))
)
```

## Common options

| Flag | Effect |
|---|---|
| `-d`, `--dump-sygus` | Print the generated SyGuS problem |
| `--bisim=false` | Skip bisimulation quotienting |
| `-v`, `-vv` | INFO / DEBUG logging |

**Experts only:**

| Flag | Effect |
|---|---|
| `--dump-chc` | Print the CHC (HORN) encoding |
| `--sygus-stats` | Print `SYGUS_STATS<TAB>ms<TAB>bytes<TAB>lines<TAB>max-bv-width` |
| `--solver <PATH>` | Use an external SyGuS solver (e.g. `cvc5`) |
| `--cegis[=N]` | CEGIS subset solving (external solver only) |

Run `loom --help` for the full list.

## Docker

```bash
docker build -t loom .
docker run --rm -v "$PWD:/work" -w /work loom benchmark/Reg/0003.sl
```

The image includes `cvc5`. `make docker` exports it to `dist/loom.tar.gz`.
