(require "theories/learning_switch-c3b0962f.sl")

(define-model P0
  (define-univ node (@node_0))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N1
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N2
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_1 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N3
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_0) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model P4
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_0 @node_1) true)
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_1 @node_0) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N5
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_1) true)
    ((@node_1 @node_0 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N6
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_1 @node_0) true)
    ((@node_1 @node_1 @node_0) true)
    ((_ _ _) false)
  )))
)

(define-model N7
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N8
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_1 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
)

(define-model N9
  (define-univ node (@node_0 @node_1 @node_2))
  (define-univ packet (@uc_packet_0))
  (interp-fun dst (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@uc_packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((@node_0 @node_0 @node_0) true)
    ((@node_0 @node_1 @node_1) true)
    ((@node_0 @node_2 @node_2) true)
    ((@node_1 @node_0 @node_0) true)
    ((@node_1 @node_0 @node_1) true)
    ((@node_1 @node_0 @node_2) true)
    ((@node_1 @node_1 @node_1) true)
    ((@node_1 @node_2 @node_2) true)
    ((@node_2 @node_0 @node_0) true)
    ((@node_2 @node_1 @node_1) true)
    ((@node_2 @node_2 @node_2) true)
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N3 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P4 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N6 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N7 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N8 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N9 fol.bot fol.bot))))

(check-synth)
