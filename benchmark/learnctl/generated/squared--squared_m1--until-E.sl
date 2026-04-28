(require "../theories/ctl-until-e-ed1c48c0.sl")
(require "../common/models/squared.sl")
(require "../common/models/squared_m1.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared st_q0 $22)))
(constraint (not (CTL.Sem phi (CTL.Asp squared_m1 st_q0 $2))))

(check-synth)
