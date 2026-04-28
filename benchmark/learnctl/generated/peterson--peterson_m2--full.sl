(require "../theories/ctl-full-9e1916e6.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m2.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $47)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m2 st_q0 $42))))

(check-synth)
