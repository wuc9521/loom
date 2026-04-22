(require "theories/fast_paxos-46d74de9.sl")
(require "../common/models/fast_paxos-c11.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))

(check-synth)
