(require "theories/fast_paxos_epr-8ffc49d2.sl")
(require "../common/models/fast_paxos_epr-c5.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))
(constraint (FO.Sem0 phi (FoAsp0 P9)))

(check-synth)
