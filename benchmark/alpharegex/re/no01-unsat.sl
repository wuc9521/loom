(require "./theories/main.sl")
(require "../common/models/no01.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (size (Pos p01)))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))

(check-synth)
