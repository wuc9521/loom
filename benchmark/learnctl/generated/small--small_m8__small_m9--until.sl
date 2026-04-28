(require "../theories/ctl-until-26cc0402.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m8.sl")
(require "../common/models/small_m9.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m8 st_i $4))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m9 st_i $4))))

(check-synth)
