(require "theories/alternating_bit_protocol-232f7bb5.sl")
(require "../common/models/alternating_bit_protocol-abp_ack_msg.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (FO.Sem0 phi (FoAsp0 P5)))

(check-synth)
