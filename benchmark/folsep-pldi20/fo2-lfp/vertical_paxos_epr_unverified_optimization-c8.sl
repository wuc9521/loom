(require "theories/vertical_paxos_epr_unverified_optimization-2bc0d415.sl")
(require "../common/models/vertical_paxos_epr_unverified_optimization-c8.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))

(check-synth)
