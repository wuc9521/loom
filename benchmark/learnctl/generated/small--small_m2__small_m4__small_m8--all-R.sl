(require "../theories/ctl-all-5d4ae448.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m2.sl")
(require "../common/models/small_m4.sl")
(require "../common/models/small_m8.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $6)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m2 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m4 st_i $7))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m8 st_i $7))))

(check-synth)
