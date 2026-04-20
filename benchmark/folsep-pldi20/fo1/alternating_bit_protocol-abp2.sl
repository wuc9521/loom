(require "theories/alternating_bit_protocol-232f7bb5.sl")
(require "../common/models/alternating_bit_protocol-abp2.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))

(check-synth)
