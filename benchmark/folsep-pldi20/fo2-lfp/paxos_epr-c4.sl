(require "theories/paxos_epr-5637217b.sl")
(require "../common/models/paxos_epr-c4.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (FO.Sem0 phi (FoAsp0 P4)))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))

(check-synth)
