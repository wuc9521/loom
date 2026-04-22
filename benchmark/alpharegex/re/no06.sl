(require "./theories/main.sl")
(require "../common/models/no06.sl")

(synth-fun r () RegLan)

(constraint (RegLan.Sem r (ReAsp p___ 0 (size (Pos p___)))))
(constraint (RegLan.Sem r (ReAsp p______ 0 (size (Pos p______)))))
(constraint (not (RegLan.Sem r (ReAsp n? 0 (size (Pos n?))))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (size (Pos n??))))))
(constraint (not (RegLan.Sem r (ReAsp n???? 0 (size (Pos n????))))))

(check-synth)
