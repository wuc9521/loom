(require "../theories/ctl-full-9e1916e6.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $54)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m6 st_q0 $78))))

(check-synth)
