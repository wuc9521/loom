(require "theories/flexible_paxos_epr-12d2d6cd.sl")
(require "../common/models/flexible_paxos_epr-c.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))

(check-synth)
