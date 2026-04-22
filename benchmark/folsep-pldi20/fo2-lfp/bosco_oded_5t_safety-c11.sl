(require "theories/shared-a2d4218b.sl")
(require "../common/models/bosco_oded_5t_safety-c11.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (not (FO.Sem0 phi (FoAsp0 N3))))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (FO.Sem0 phi (FoAsp0 P7)))
(constraint (not (FO.Sem0 phi (FoAsp0 N8))))

(check-synth)
