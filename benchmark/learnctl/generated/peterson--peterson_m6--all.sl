(require "../theories/ctl-all-771cdd14.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $47)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m6 st_q0 $18))))

(check-synth)
