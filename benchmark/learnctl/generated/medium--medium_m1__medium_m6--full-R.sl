(require "../theories/ctl-full-ee23b830.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m1.sl")
(require "../common/models/medium_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $26)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $26)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q13 $32))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q0 $32))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q13 $28))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q0 $28))))

(check-synth)
