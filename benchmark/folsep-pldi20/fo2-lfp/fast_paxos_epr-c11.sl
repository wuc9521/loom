(require "theories/fast_paxos_epr-8ffc49d2.sl")
(require "../common/models/fast_paxos_epr-c11.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))

(check-synth)
