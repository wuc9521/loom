(require "../theories/ctl-full-e-aa5b1ced.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m2.sl")
(require "../common/models/medium_m8.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q13 $20))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m2 st_q0 $14))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m8 st_q13 $12))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m8 st_q0 $10))))

(check-synth)
