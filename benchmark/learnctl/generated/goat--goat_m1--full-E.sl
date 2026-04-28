(require "../theories/ctl-full-e-18429d18.sl")
(require "../common/models/goat.sl")
(require "../common/models/goat_m1.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp goat st_q0 $63)))
(constraint (not (CTL.Sem phi (CTL.Asp goat_m1 st_q0 $11))))

(check-synth)
