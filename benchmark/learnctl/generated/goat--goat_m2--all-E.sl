(require "../theories/ctl-all-e-4de641d5.sl")
(require "../common/models/goat.sl")
(require "../common/models/goat_m2.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp goat st_q0 $63)))
(constraint (not (CTL.Sem phi (CTL.Asp goat_m2 st_q0 $31))))

(check-synth)
