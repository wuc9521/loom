# External Tools And Upstream Benchmarks

This branch keeps the comparison-tool metadata and the benchmark inputs that are
not native Delis `.sl` files, so the artifact no longer depends on an old server
or an opaque Docker layer.

## Tool Repositories

- `cvc5`: <https://github.com/cvc5/cvc5>
- `folseparators`: <https://github.com/jrkoenig/folseparators>
- `mypyvy` (used by parts of the folseparators corpus): <https://github.com/wilcoxjay/mypyvy>
- `AlphaRegexPublic`: <https://github.com/kupl/AlphaRegexPublic>
- `SySLite`: <https://github.com/SySLite/SySLite>
- `samples2LTL`: <https://github.com/gergia/samples2LTL>
- `learnctl`: <https://gitlab.lre.epita.fr/adrien/learnctl/>

## Benchmark Inputs In This Repository

- `reg-alpharegex-cvc5`: `benchmark/Reg/alpharegex/sygus/*.sl`
- `reg-deepregex-cvc5`: `benchmark/Reg/deepregex/sygus/*.sl`
- `reg-so-cvc5`: `benchmark/Reg/so/sygus/*.sl`
- `fo-folseparators-artifact`: `benchmark-upstream/FO/folseparators/conjectures/extracted/*.fol`
- `fo-folseparators-artifact` metadata: `benchmark-upstream/FO/folseparators/conjectures/benchmark.json`
- `fo-folseparators` raw mypyvy-origin conjectures: `benchmark-upstream/FO/folseparators/conjectures/mypyvy/*.fol`
- `ctl-learnctl`: translated Delis inputs already live under `benchmark/CTL/learnctl/generated/*.sl`

## Notes

- The folseparators upstream corpus was recovered from the uploaded Docker image
  archive `560.tgz`, specifically the layer that contained `/root/folseparators/conjectures/`.
- The repository intentionally vendors only benchmark inputs and metadata here,
  not the external tool source trees themselves.
- Remote setup is scripted in `scripts/setup-external-tools.sh`.
- External comparisons runnable through `delis-bench` are listed in
  `benchmarks-external.toml`.
