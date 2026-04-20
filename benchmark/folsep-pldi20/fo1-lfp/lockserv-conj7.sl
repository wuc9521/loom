(require "theories/lockserv-93e3d996.sl")
(require "../common/models/lockserv-conj7.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (FO.Sem0 phi (FoAsp0 P1)))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))

(check-synth)
