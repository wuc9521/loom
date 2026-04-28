(require "../theories/ctl-full-ee23b830.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m1.sl")
(require "../common/models/medium_m5.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $26)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $26)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q13 $32))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q0 $32))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q13 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q0 $29))))

(check-synth)
