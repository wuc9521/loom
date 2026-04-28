(require "../theories/ctl-until-e-751b2ba9.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m3.sl")
(require "../common/models/small_m8.sl")
(require "../common/models/small_m9.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m3 st_i $5))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m8 st_i $4))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m9 st_i $4))))

(check-synth)
