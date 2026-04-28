(require "../theories/ctl-all-e-06bdbfd0.sl")
(require "../common/models/squared.sl")
(require "../common/models/squared_m2.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared st_q0 $22)))
(constraint (not (CTL.Sem phi (CTL.Asp squared_m2 st_q0 $10))))

(check-synth)
