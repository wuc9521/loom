(require "theories/alternating_bit_protocol-232f7bb5.sl")
(require "../common/models/alternating_bit_protocol-abp1.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))

(check-synth)
