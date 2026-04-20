(require "../signatures/tlb_pcrel-838d0912.sl")
(provide P0 N1 N2 N3 N4 N5 N6 N7 P8 P9 P10 P11 P12 P13 P14 P15 P16)

(define-model P0
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N1
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N2
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N3
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N4
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N5
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N6
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model N7
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P8
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P9
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P10
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P11
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P12
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P13
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_1))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((@tlb.processor_1) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P14
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P15
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)

(define-model P16
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @tlb.pmap_0)
    ((_) @tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((_) false)
  )))
  (interp-fun tlb.actionneeded (match (
    ((_) false)
  )))
  (interp-fun tlb.active (match (
    ((_) false)
  )))
  (interp-fun tlb.currentcpu (match (
    ((_ _) false)
  )))
  (interp-fun tlb.error (false))
  (interp-fun tlb.interrupt (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_b1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i1 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i11 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_i12 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i14 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i15 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i4 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i8 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_i9 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_m2 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_m5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r1 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r2 (match (
    ((@tlb.processor_0) true)
    ((_) false)
  )))
  (interp-fun tlb.pc_r3 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r5 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r6 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r7 (match (
    ((_) false)
  )))
  (interp-fun tlb.pc_r8 (match (
    ((_) false)
  )))
  (interp-fun tlb.plock (match (
    ((_) false)
  )))
  (interp-fun tlb.scheduled (match (
    ((_) false)
  )))
  (interp-fun tlb.todo (match (
    ((_ _) false)
  )))
)
