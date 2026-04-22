(require "theories/chord-2f2f7007.sl")
(require "../common/models/chord-1s-nofail-liveness-epr-all-conjectures-chord.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))

(check-synth)
