(require "theories/leader_election_ring_btw_nonreflexive-c4d3b61d.sl")
(require "../common/models/leader_election_ring_btw_nonreflexive-c.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))

(check-synth)
