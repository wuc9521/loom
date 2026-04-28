(require "../theories/ctl-all-00dc3888.sl")
(require "../common/models/goat.sl")
(require "../common/models/goat_m4.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp goat st_q0 $63)))
(constraint (not (CTL.Sem phi (CTL.Asp goat_m4 st_q0 $63))))

(check-synth)
