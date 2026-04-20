(require "theories/ticket_nested-ab5662a6.sl")
(require "../common/models/ticket_nested-ticket_protocol11.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (FO.Sem0 phi (FoAsp0 P8)))
(constraint (FO.Sem0 phi (FoAsp0 P9)))
(constraint (not (FO.Sem0 phi (FoAsp0 N10))))

(check-synth)
