(require "theories/chord-2f2f7007.sl")
(require "../common/models/chord-1s-nofail-liveness-epr-chord10.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (FO.Sem0 phi (FoAsp0 P8)))
(constraint (FO.Sem0 phi (FoAsp0 P9)))

(check-synth)
