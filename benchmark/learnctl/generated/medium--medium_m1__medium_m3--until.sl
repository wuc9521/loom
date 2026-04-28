(require "../theories/ctl-until-ea7be407.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m1.sl")
(require "../common/models/medium_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q13 $15))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m1 st_q0 $15))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m3 st_q13 $20))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m3 st_q0 $15))))

(check-synth)
