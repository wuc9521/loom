(require "theories/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")
(require "../common/models/vertical_paxos_epr_unverified_optimization-c6.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (FO.Sem0 phi (FoAsp0 P8)))

(check-synth)
