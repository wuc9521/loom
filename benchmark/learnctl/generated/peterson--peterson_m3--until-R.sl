(require "../theories/ctl-until-fea6804f.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $54)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m3 st_q0 $86))))

(check-synth)
