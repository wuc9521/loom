(require "../theories/ctl-until-e-1b9ea15a.sl")
(require "../common/models/peterson2.sl")
(require "../common/models/peterson2_m4.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson2 st_q0 $119)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson2_m4 st_q0 $21))))

(check-synth)
