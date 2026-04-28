(require "../theories/ctl-until-26cc0402.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m1.sl")
(require "../common/models/small_m2.sl")
(require "../common/models/small_m5.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $6)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m1 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m2 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m5 st_i $7))))

(check-synth)
