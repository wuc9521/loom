(require "../theories/ctl-until-e-ed1c48c0.sl")
(require "../common/models/squared31.sl")
(require "../common/models/squared31_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared31 st_q0 $64)))
(constraint (not (CTL.Sem phi (CTL.Asp squared31_m3 st_q0 $33))))

(check-synth)
