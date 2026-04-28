(require "../theories/ctl-all-bb52f665.sl")
(require "../common/models/medium.sl")
(require "../common/models/medium_m5.sl")
(require "../common/models/medium_m7.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp medium st_q0 $12)))
(constraint (CTL.Sem phi (CTL.Asp medium st_q13 $12)))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q13 $21))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m5 st_q0 $18))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q13 $16))))
(constraint (not (CTL.Sem phi (CTL.Asp medium_m7 st_q0 $17))))

(check-synth)
