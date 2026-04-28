(require "../theories/ctl-all-e-395803cd.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m2.sl")
(require "../common/models/medium_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q13 $20))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q0 $14))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q13 $16))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q0 $18))))

(check-synth)
