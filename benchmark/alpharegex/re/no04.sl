(require "./theories/main.sl")
(require "../common/models/no04.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p01 0 (size (Pos p01)))))
(constraint (RegLan.Sem r (ReAsp p0_1 0 (size (Pos p0_1)))))
(constraint (RegLan.Sem r (ReAsp p0__1 0 (size (Pos p0__1)))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n0 0 (size (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n00 0 (size (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n0?0 0 (size (Pos n0?0))))))
(constraint (not (RegLan.Sem r (ReAsp n1?1 0 (size (Pos n1?1))))))
(constraint (not (RegLan.Sem r (ReAsp n1?0 0 (size (Pos n1?0))))))

(check-synth)
