(require "theories/ticket-4357690c.sl")
(require "../common/models/ticket-ticket_protocol_tp1_impl.sl")

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
(constraint (not (FO.Sem0 phi (FoAsp0 N9))))
(constraint (FO.Sem0 phi (FoAsp0 P10)))

(check-synth)
