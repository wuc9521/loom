(require "./theories/main.sl")
(require "../common/models/no12.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p000 0 (size (Pos p000)))))
(constraint (RegLan.Sem r (ReAsp p00011 0 (size (Pos p00011)))))
(constraint (RegLan.Sem r (ReAsp p0001111 0 (size (Pos p0001111)))))
(constraint (RegLan.Sem r (ReAsp p000011 0 (size (Pos p000011)))))
(constraint (RegLan.Sem r (ReAsp p00001111 0 (size (Pos p00001111)))))
(constraint (RegLan.Sem r (ReAsp p00000111111 0 (size (Pos p00000111111)))))
(constraint (not (RegLan.Sem r (ReAsp n0 0 (size (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n00 0 (size (Pos n00))))))
(constraint (not (RegLan.Sem r (ReAsp n0001 0 (size (Pos n0001))))))
(constraint (not (RegLan.Sem r (ReAsp n00001 0 (size (Pos n00001))))))
(constraint (not (RegLan.Sem r (ReAsp n0000111 0 (size (Pos n0000111))))))

(check-synth)
