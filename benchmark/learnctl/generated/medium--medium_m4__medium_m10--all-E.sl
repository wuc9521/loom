(require "../theories/ctl-all-e-395803cd.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m4.sl")
(require "../common/models/medium_m10.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m4 st_q13 $15))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m4 st_q0 $5))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m10 st_q13 $13))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m10 st_q0 $10))))

(check-synth)
