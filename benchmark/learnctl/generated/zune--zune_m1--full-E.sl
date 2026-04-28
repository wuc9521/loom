(require "../theories/ctl-full-e-97925d19.sl")
(require "../common/models/zune.sl")
(require "../common/models/zune_m1.sl")

(synth-fun phi () CTL)

(constraint (CTL.Sem phi (CTL.Asp zune st_q0 $91)))
(constraint (not (CTL.Sem phi (CTL.Asp zune_m1 st_q0 $91))))

(check-synth)
