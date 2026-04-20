(require "theories/ticket_nested-ab5662a6.sl")
(require "../common/models/ticket_nested-ticket_protocol4.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (FO.Sem0 phi (FoAsp0 P8)))
(constraint (FO.Sem0 phi (FoAsp0 P9)))
(constraint (FO.Sem0 phi (FoAsp0 P10)))

(check-synth)
