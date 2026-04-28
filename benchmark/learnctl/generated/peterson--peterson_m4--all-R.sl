(require "../theories/ctl-all-771cdd14.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m4.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $54)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m4 st_q0 $38))))

(check-synth)
