(require "../theories/ctl-full-f6aab77a.sl")
(require "../common/models/squared.sl")
(require "../common/models/squared_m1.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared st_q0 $87)))
(constraint (not (CTL.Sem phi (CTL.Asp squared_m1 st_q0 $2))))

(check-synth)
