(require "theories/ticket_nested-ab5662a6.sl")
(require "../common/models/ticket_nested-ticket_protocol16.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))

(check-synth)
