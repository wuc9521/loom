(require "../theories/ctl-until-fea6804f.sl")
(require "../common/models/peterson.sl")
(require "../common/models/peterson_m5.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp peterson st_q0 $47)))
(constraint (not (CTL.Sem phi (CTL.Asp peterson_m5 st_q0 $31))))

(check-synth)
