(require "../theories/ctl-until-e-751b2ba9.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m3.sl")
(require "../common/models/small_m7.sl")
(require "../common/models/small_m10.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m3 st_i $5))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m7 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m10 st_i $6))))

(check-synth)
