(require "theories/tlb_pcrel-838d0912.sl")

(define-model P0
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@uc_tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@uc_tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@uc_tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((@tlb.processor_0) true)
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

(define-model N1
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@uc_tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@uc_tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@uc_tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((@tlb.processor_0) true)
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
  (define-univ tlb.pmap (@uc_tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0 @tlb.processor_1))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@uc_tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@uc_tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((@tlb.processor_1) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((@tlb.processor_1) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((@tlb.processor_1) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.actionlock (match (
    ((@tlb.processor_0) true)
    ((@tlb.processor_1) true)
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
    ((@tlb.processor_1) true)
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

(define-model P3
  (define-univ tlb.pageentry (@uc_tlb.pageentry_0))
  (define-univ tlb.pmap (@uc_tlb.pmap_0))
  (define-univ tlb.processor (@tlb.processor_0))
  (interp-fun tlb.tp1.sk0 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk1 (@tlb.processor_0))
  (interp-fun tlb.tp1.sk2 (@uc_tlb.pmap_0))
  (interp-fun tlb.pentry (match (
    ((@uc_tlb.pmap_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.tlb (match (
    ((@tlb.processor_0) @uc_tlb.pageentry_0)
    ((_) @uc_tlb.pageentry_0)
  )))
  (interp-fun tlb.userpmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
  )))
  (interp-fun tlb.writepmap (match (
    ((@tlb.processor_0) @uc_tlb.pmap_0)
    ((_) @uc_tlb.pmap_0)
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

(synth-fun phi () Formula)

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P3 fol.bot fol.bot)))

(check-synth)
