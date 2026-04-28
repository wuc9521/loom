(require "../theories/ctl-all-771cdd14.sl")
(require "../common/models/peterson2.sl")
(require "../common/models/peterson2_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson2 st_q0 $134)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson2_m3 st_q0 $134))))

(check-synth)
