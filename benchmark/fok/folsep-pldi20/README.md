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

Attempt: recover the synthesized formula from use z3 `(get-model)`

```bash
(base) *[main][~/Codes/Rust/delis]$ z3 /tmp/alternating_bit_protocol-abp_ack_msg.smt2 | vim -
Vim: Reading from stdin...
```

```SMT-Lib
; (base) *[main][~/Codes/Rust/delis]$ time delis ./benchmark/fok/folsep-pldi20/bosco_oded_3t_safety-c0.sl --dump-sygus
(define-fun fol.var.x1.value () (_ BitVec 428) (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00 (concat #b1 (concat #b0000000000 (concat #b1 (concat #b00000 (concat #b1 (concat #b00000 (concat #b1 (concat #b00000 (concat #b1 (concat #b00000 (concat #b1 (concat #b00000 (concat #b1 #b00000))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))
(synth-fun phi () (_ BitVec 464)  ((Start (_ BitVec 464)) (Term.node (_ BitVec 232)) (Term.quorum_a (_ BitVec 232)) (Term.quorum_b (_ BitVec 232)) (Term.quorum_c (_ BitVec 232)) (Term.value (_ BitVec 428))) ((Start (_ BitVec 464) (fol.true fol.false (fol.not Start) (fol.and Start Start) (fol.or Start Start) (fol.forall.x0.node Start) (fol.exists.x0.node Start) (fol.forall.x1.node Start) (fol.exists.x1.node Start) (fol.eq.node Term.node Term.node) (fol.forall.x0.quorum_a Start) (fol.exists.x0.quorum_a Start) (fol.forall.x1.quorum_a Start) (fol.exists.x1.quorum_a Start) (fol.eq.quorum_a Term.quorum_a Term.quorum_a) (fol.forall.x0.quorum_b Start) (fol.exists.x0.quorum_b Start) (fol.forall.x1.quorum_b Start) (fol.exists.x1.quorum_b Start) (fol.eq.quorum_b Term.quorum_b Term.quorum_b) (fol.forall.x0.quorum_c Start) (fol.exists.x0.quorum_c Start) (fol.forall.x1.quorum_c Start) (fol.exists.x1.quorum_c Start) (fol.eq.quorum_c Term.quorum_c Term.quorum_c) (fol.forall.x0.value Start) (fol.exists.x0.value Start) (fol.forall.x1.value Start) (fol.exists.x1.value Start) (fol.eq.value Term.value Term.value) (fol.atom.dec Term.node Term.value) (fol.atom.done Term.node) (fol.atom.input Term.node Term.value) (fol.atom.member_a Term.node Term.quorum_a) (fol.atom.member_b Term.node Term.quorum_b) (fol.atom.member_c Term.node Term.quorum_c) (fol.atom.member_f Term.node) (fol.atom.rcv_msg Term.node Term.node Term.value) (fol.atom.rcv_msg_proj Term.node Term.node) (fol.atom.sent_msg Term.node Term.node Term.value) (fol.atom.sent_msg_proj Term.node Term.node) (fol.atom.und_cons Term.node Term.value) (fol.atom.und_cons_dec Term.value))) (Term.node (_ BitVec 232) (fol.var.x0.node fol.var.x1.node)) (Term.quorum_a (_ BitVec 232) (fol.var.x0.quorum_a fol.var.x1.quorum_a)) (Term.quorum_b (_ BitVec 232) (fol.var.x0.quorum_b fol.var.x1.quorum_b)) (Term.quorum_c (_ BitVec 232) (fol.var.x0.quorum_c fol.var.x1.quorum_c)) (Term.value (_ BitVec 428) (fol.var.x0.value fol.var.x1.value))))
(constraint (= ((_ extract 232 232) phi) #b1))
(constraint (= ((_ extract 0 0) phi) #b1))
(constraint (= ((_ extract 36 36) phi) #b0))
(constraint (= ((_ extract 85 85) phi) #b1))
(constraint (= ((_ extract 134 134) phi) #b0))
(constraint (= ((_ extract 183 183) phi) #b1))
(check-synth)
; delis ./benchmark/fok/folsep-pldi20/bosco_oded_3t_safety-c0.sl --dump-sygus  0.27s user 0.02s system 81% cpu 0.353 total
```
