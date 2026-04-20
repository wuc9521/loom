(require "theories/lockserv-93e3d996.sl")
(require "../common/models/lockserv-conj3.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (FO.Sem0 phi (FoAsp0 P2)))

(check-synth)
