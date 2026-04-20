(require "theories/chord-2f2f7007.sl")
(require "../common/models/chord-1s-nofail-liveness-epr-chord_at_most_one_ring.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))

(check-synth)
