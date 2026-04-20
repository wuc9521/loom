(require "theories/fast_paxos-46d74de9.sl")
(require "../common/models/fast_paxos-c8.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (FO.Sem0 phi (FoAsp0 P8)))
(constraint (not (FO.Sem0 phi (FoAsp0 N9))))
(constraint (not (FO.Sem0 phi (FoAsp0 N10))))

(check-synth)
