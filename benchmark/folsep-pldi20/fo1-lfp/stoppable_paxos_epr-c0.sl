(require "theories/stoppable_paxos_epr-d7674849.sl")
(require "../common/models/stoppable_paxos_epr-c0.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))

(check-synth)
