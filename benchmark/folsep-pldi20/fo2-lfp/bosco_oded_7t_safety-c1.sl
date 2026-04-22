(require "theories/shared-a2d4218b.sl")
(require "../common/models/bosco_oded_7t_safety-c1.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))

(check-synth)
