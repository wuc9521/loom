(require "../theories/ctl-full-ea0fd77f.sl")
(require "../common/models/goat.sl")
(require "../common/models/goat_m5.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp goat st_q0 $63)))
(constraint (not (CTL.Sem phi (CTL.Asp goat_m5 st_q0 $63))))

(check-synth)
