(require "../theories/ctl-all-e-7200bbf4.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m3.sl")
(require "../common/models/small_m4.sl")
(require "../common/models/small_m10.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m3 st_i $5))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m4 st_i $4))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m10 st_i $6))))

(check-synth)
