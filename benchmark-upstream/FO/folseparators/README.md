# Folseparators Upstream Corpus

This directory preserves the original folseparators benchmark inputs that were
used by the PLDI'20 artifact.

Provenance:

- Recovered on 2026-05-29 from `/Users/wuchentian/Downloads/artifact-upload/560.tgz`
- Source inside the Docker image: `/root/folseparators/conjectures/`
- The canonical artifact index is `conjectures/benchmark.json`

Contents:

- `conjectures/extracted/*.fol`: the benchmark cases referenced by `benchmark.json`
- `conjectures/mypyvy/*.fol`: mypyvy-derived conjecture files that were also
  bundled in the artifact image

Only source benchmark files are stored here. Tool source code remains external.
