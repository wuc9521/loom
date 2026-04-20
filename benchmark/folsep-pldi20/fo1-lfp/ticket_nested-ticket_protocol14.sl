(require "theories/ticket_nested-ab5662a6.sl")
(require "../common/models/ticket_nested-ticket_protocol14.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))
(constraint (FO.Sem0 phi (FoAsp0 P9)))
(constraint (not (FO.Sem0 phi (FoAsp0 N10))))

(check-synth)
