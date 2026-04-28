(require "../theories/ctl-full-e-ba485660.sl")
(require "../common/models/squared31.sl")
(require "../common/models/squared31_m1.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared31 st_q0 $64)))
(constraint (not (CTL.Sem phi (CTL.Asp squared31_m1 st_q0 $2))))

(check-synth)
