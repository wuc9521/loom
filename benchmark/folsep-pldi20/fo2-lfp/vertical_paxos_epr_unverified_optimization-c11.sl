(require "theories/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")
(require "../common/models/vertical_paxos_epr_unverified_optimization-c11.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))
(constraint (not (FO.Sem0 phi (FoAsp0 N9))))

(check-synth)
