(require "../theories/ctl-full-4ff7af62.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m5.sl")
(require "../common/models/small_m6.sl")
(require "../common/models/small_m7.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m5 st_i $5))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m6 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m7 st_i $6))))

(check-synth)
