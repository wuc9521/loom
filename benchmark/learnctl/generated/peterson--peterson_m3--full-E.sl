(require "../theories/ctl-full-e-437ce0ff.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $47)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m3 st_q0 $80))))

(check-synth)
