(require "theories/chord-2f2f7007.sl")
(require "../common/models/chord-1s-nofail-liveness-epr-all-conjectures-chord_at_least_one_ring.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (FO.Sem0 phi (FoAsp0 P3)))

(check-synth)
