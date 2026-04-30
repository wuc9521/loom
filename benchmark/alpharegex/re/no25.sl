(require "./theories/main.sl")
(require "../common/models/no25.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (size (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p101010 0 (size (Pos p101010)))))
(constraint (RegLan.Sem r (ReAsp p00011000 0 (size (Pos p00011000)))))
(constraint (RegLan.Sem r (ReAsp p00100110001 0 (size (Pos p00100110001)))))
(constraint (not (RegLan.Sem r (ReAsp n111 0 (size (Pos n111))))))
(constraint (not (RegLan.Sem r (ReAsp n110011 0 (size (Pos n110011))))))
(constraint (not (RegLan.Sem r (ReAsp n0110110 0 (size (Pos n0110110))))))
(constraint (not (RegLan.Sem r (ReAsp n00011001100 0 (size (Pos n00011001100))))))
(constraint (not (RegLan.Sem r (ReAsp n011100011110 0 (size (Pos n011100011110))))))

(check-synth)
