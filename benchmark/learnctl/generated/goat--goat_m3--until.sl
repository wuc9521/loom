(require "../theories/ctl-until-12893879.sl")
(require "../common/models/goat.sl")
(require "../common/models/goat_m3.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp goat st_q0 $63)))
(constraint (not (CTL.Sem phi (CTL.Asp goat_m3 st_q0 $31))))

(check-synth)
