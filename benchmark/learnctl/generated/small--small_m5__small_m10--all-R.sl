(require "../theories/ctl-all-5d4ae448.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m5.sl")
(require "../common/models/small_m10.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $6)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m5 st_i $7))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m10 st_i $7))))

(check-synth)
