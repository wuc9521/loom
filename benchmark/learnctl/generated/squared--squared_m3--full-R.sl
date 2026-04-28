(require "../theories/ctl-full-f6aab77a.sl")
(require "../common/models/squared.sl")
(require "../common/models/squared_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp squared st_q0 $87)))
(constraint (not (CTL.Sem phi (CTL.Asp squared_m3 st_q0 $32))))

(check-synth)
