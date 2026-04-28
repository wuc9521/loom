(require "../theories/ctl-full-4ff7af62.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m2.sl")
(require "../common/models/small_m5.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m2 st_i $4))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m5 st_i $5))))

(check-synth)
