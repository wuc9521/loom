(require "../theories/ctl-all-bb52f665.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m2.sl")
(require "../common/models/medium_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $26)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $26)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q13 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q0 $29))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q13 $28))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q0 $28))))

(check-synth)
