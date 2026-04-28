(require "../theories/ctl-all-bb52f665.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m5.sl")
(require "../common/models/medium_m7.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $26)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $26)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q13 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q0 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q13 $30))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q0 $30))))

(check-synth)
