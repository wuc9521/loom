(require "theories/learning_switch-c3b0962f.sl")

(define-model P0
  (define-univ node (@node_0))
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
    ((@packet_0 @node_0 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
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
    ((@packet_0) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N2
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N3
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N4
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
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
    ((@packet_0) @node_0)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model P6
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N8
  (define-univ node (@node_0 @node_1))
  (define-univ packet (@packet_0 @packet_1))
  (interp-fun dst (match (
    ((@packet_0) @node_0)
    ((@packet_1) @node_0)
    ((_) @node_0)
  )))
  (interp-fun src (match (
    ((@packet_0) @node_0)
    ((@packet_1) @node_1)
    ((_) @node_0)
  )))
  (interp-fun link (match (
    ((_ _) false)
  )))
  (interp-fun pending (match (
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((@packet_1 @node_0 @node_0) true)
    ((@packet_1 @node_0 @node_1) true)
    ((@packet_1 @node_1 @node_0) true)
    ((@packet_1 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model P9
  (define-univ node (@node_0))
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
    ((@packet_0 @node_0 @node_0) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model P10
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
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N11
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
    ((@packet_0 @node_0 @node_0) true)
    ((@packet_0 @node_0 @node_1) true)
    ((@packet_0 @node_1 @node_0) true)
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N12
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
    ((@packet_0 @node_1 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((@node_1 @node_0) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(define-model N13
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
    ((@packet_0 @node_0 @node_1) true)
    ((_ _ _) false)
  )))
  (interp-fun route.dom (match (
    ((@node_0 @node_0) true)
    ((@node_0 @node_1) true)
    ((@node_1 @node_1) true)
    ((_ _) false)
  )))
  (interp-fun route.tc (match (
    ((_ _ _) false)
  )))
)

(synth-fun phi () Formula)

(constraint (FO.Valid phi (FoAsp P0 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P0 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N1 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N2 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N3 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N4 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N5 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P6 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N7 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N8 fol.bot fol.bot))))
(constraint (FO.Sem phi (FoAsp P9 fol.bot fol.bot)))
(constraint (FO.Sem phi (FoAsp P10 fol.bot fol.bot)))
(constraint (not (FO.Sem phi (FoAsp N11 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N12 fol.bot fol.bot))))
(constraint (not (FO.Sem phi (FoAsp N13 fol.bot fol.bot))))

(check-synth)
