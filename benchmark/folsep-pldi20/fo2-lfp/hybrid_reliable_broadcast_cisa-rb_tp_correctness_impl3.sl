(require "theories/hybrid_reliable_broadcast_cisa-ad5dfe37.sl")
(require "../common/models/hybrid_reliable_broadcast_cisa-rb_tp_correctness_impl3.sl")

(synth-fun phi () Formula0)

(constraint (FO.Sem0 phi (FoAsp0 P0)))
(constraint (not (FO.Sem0 phi (FoAsp0 N1))))

(check-synth)
