(require "../theories/ctl-until-e-cb3f35bf.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m7.sl")
(require "../common/models/medium_m8.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q13 $16))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q0 $17))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m8 st_q13 $12))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m8 st_q0 $10))))

(check-synth)
