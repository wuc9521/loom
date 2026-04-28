(require "../theories/ctl-full-4ff7af62.sl")
(require "../common/models/small.sl")
(require "../common/models/small_m1.sl")
(require "../common/models/small_m4.sl")
(require "../common/models/small_m9.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp small st_i $6)))
(constraint (not (CTL.Sem phi (CTL.Asp small_m1 st_i $6))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m4 st_i $7))))
(constraint (not (CTL.Sem phi (CTL.Asp small_m9 st_i $6))))

(check-synth)
