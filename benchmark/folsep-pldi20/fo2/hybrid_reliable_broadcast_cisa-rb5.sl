(require "theories/hybrid_reliable_broadcast_cisa-ad5dfe37.sl")
(require "../common/models/hybrid_reliable_broadcast_cisa-rb5.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))
(constraint (not (FO.Sem0 phi (FoAsp0 N9))))
(constraint (not (FO.Sem0 phi (FoAsp0 N10))))
(constraint (not (FO.Sem0 phi (FoAsp0 N11))))

(check-synth)
