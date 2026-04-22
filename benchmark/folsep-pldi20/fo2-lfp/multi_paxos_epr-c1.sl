(require "theories/multi_paxos_epr-ed65a58f.sl")
(require "../common/models/multi_paxos_epr-c1.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))

(check-synth)
