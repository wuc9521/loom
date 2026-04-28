(require "../theories/ctl-all-e-5b426fe2.sl")
(require "../common/models/peterson2.sl")
(require "../common/models/peterson2_m6.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson2 st_q0 $119)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson2_m6 st_q0 $112))))

(check-synth)
