<https://dl.acm.org/do/10.1145/3395650/full/>

```bash
(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/fok/folseparators/chord-1s-nofail-liveness-epr-all-conjectures-chord_bs.sl
(
(define-fun phi () Formula (fol.forall.x0.chord.node (fol.or (fol.eq.chord.node fol.var.x0.chord.node fol.var.x1.chord.node) (fol.and (fol.eq.chord.node fol.var.x0.chord.node fol.term.chord.tp1.n0) (fol.atom.chord.bs.p fol.var.x1.chord.node fol.var.x0.chord.node fol.var.x0.chord.node)))))
)
delis   14.71s user 0.15s system 99% cpu 14.956 total
```

```bash
(base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/fok/folsep-pldi20/tlb_pcrel-tlb.sl
(
(define-fun phi () Formula (fol.forall.x0.tlb.pmap (fol.or (fol.atom.tlb.plock fol.var.x0.tlb.pmap) (fol.not (fol.or (fol.atom.tlb.pc_i11 fol.var.x0.tlb.processor) (fol.or (fol.atom.tlb.pc_i12 fol.var.x0.tlb.processor) (fol.atom.tlb.pc_i14 fol.var.x0.tlb.processor)))))))
)
delis ./benchmark/fok/folsep-pldi20/tlb_pcrel-tlb.sl  75.08s user 0.34s system 99% cpu 1:15.55 total
```
