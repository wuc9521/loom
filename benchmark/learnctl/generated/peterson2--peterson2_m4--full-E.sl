(require "../theories/ctl-full-e-437ce0ff.sl")
(require "../common/models/peterson2.sl")
(require "../common/models/peterson2_m4.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson2 st_q0 $119)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson2_m4 st_q0 $21))))

(check-synth)
