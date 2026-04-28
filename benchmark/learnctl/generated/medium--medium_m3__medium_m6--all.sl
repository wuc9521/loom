(require "../theories/ctl-all-bb52f665.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m3.sl")
(require "../common/models/medium_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m3 st_q13 $20))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m3 st_q0 $15))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q13 $16))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m6 st_q0 $18))))

(check-synth)
