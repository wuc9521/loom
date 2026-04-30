(require "./theories/main.sl")
(require "../common/models/no21.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p00 0 (size (Pos p00)))))
(constraint (RegLan.Sem r (ReAsp p1001 0 (size (Pos p1001)))))
(constraint (RegLan.Sem r (ReAsp p010010 0 (size (Pos p010010)))))
(constraint (RegLan.Sem r (ReAsp p1011001110 0 (size (Pos p1011001110)))))
(constraint (not (RegLan.Sem r (ReAsp n01 0 (size (Pos n01))))))
(constraint (not (RegLan.Sem r (ReAsp n11 0 (size (Pos n11))))))
(constraint (not (RegLan.Sem r (ReAsp n000 0 (size (Pos n000))))))
(constraint (not (RegLan.Sem r (ReAsp n00100 0 (size (Pos n00100))))))

(check-synth)
