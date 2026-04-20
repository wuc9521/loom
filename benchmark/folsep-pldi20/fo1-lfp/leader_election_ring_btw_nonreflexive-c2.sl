(require "theories/leader_election_ring_btw_nonreflexive-c4d3b61d.sl")
(require "../common/models/leader_election_ring_btw_nonreflexive-c2.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))

(check-synth)
