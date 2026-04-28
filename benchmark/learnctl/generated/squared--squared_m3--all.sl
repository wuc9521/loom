(require "../theories/ctl-all-75ce8b78.sl")
(require "../common/models/squared.sl")
(require "../common/models/squared_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared st_q0 $22)))
(constraint (not (CTL.Sem phi (CTL.Asp squared_m3 st_q0 $12))))

(check-synth)
