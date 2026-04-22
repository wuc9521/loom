(require "theories/shared-a2d4218b.sl")
(require "../common/models/bosco_oded_5t_safety-c.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (FO.Sem0 phi (FoAsp0 P5)))
(constraint (not (FO.Sem0 phi (FoAsp0 N6))))

(check-synth)
