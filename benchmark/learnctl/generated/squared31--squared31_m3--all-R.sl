(require "../theories/ctl-all-75ce8b78.sl")
(require "../common/models/squared31.sl")
(require "../common/models/squared31_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared31 st_q0 $591)))
(constraint (not (CTL.Sem phi (CTL.Asp squared31_m3 st_q0 $95))))

(check-synth)
