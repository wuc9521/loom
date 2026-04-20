(require "theories/tlb_pcrel-838d0912.sl")
(require "../common/models/tlb_pcrel-tlb23.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))
(constraint (not (FO.Sem0 phi (FoAsp0 N2))))
(constraint (FO.Sem0 phi (FoAsp0 P3)))
(constraint (not (FO.Sem0 phi (FoAsp0 N4))))
(constraint (not (FO.Sem0 phi (FoAsp0 N5))))
(constraint (FO.Sem0 phi (FoAsp0 P6)))
(constraint (not (FO.Sem0 phi (FoAsp0 N7))))
(constraint (FO.Sem0 phi (FoAsp0 P8)))

(check-synth)
