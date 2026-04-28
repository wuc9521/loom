(require "../theories/ctl-until-ea7be407.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m2.sl")
(require "../common/models/medium_m9.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $26)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $26)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q13 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q0 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m9 st_q13 $30))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m9 st_q0 $30))))

(check-synth)
