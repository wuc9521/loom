(require "../theories/ctl-all-e-7200bbf4.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m1.sl")
(require "../common/models/small_m3.sl")
(require "../common/models/small_m8.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $4)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m1 st_i $3))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m3 st_i $5))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m8 st_i $4))))

(check-synth)
