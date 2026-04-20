(require "theories/byz_paxos-0f4e99fa.sl")
(require "../common/models/byz_paxos-conj14.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))

(check-synth)
