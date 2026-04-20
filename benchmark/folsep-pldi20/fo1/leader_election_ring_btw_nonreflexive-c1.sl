(require "theories/leader_election_ring_btw_nonreflexive-c4d3b61d.sl")
(require "../common/models/leader_election_ring_btw_nonreflexive-c1.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))
(constraint (not (FO.Sem0 phi (FoAsp0 N9))))

(check-synth)
