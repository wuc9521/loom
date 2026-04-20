(require "theories/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")
(require "../common/models/vertical_paxos_epr_unverified_optimization-c.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))

(check-synth)
