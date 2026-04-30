(require "./theories/main.sl")
(require "../common/models/no05.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p__0 0 (size (Pos p__0)))))
(constraint (RegLan.Sem r (ReAsp p__0_ 0 (size (Pos p__0_)))))
(constraint (RegLan.Sem r (ReAsp p__0__ 0 (size (Pos p__0__)))))
(constraint (not (RegLan.Sem r (ReAsp n? 0 (size (Pos n?))))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (size (Pos n??))))))
(constraint (not (RegLan.Sem r (ReAsp n??1 0 (size (Pos n??1))))))
(constraint (not (RegLan.Sem r (ReAsp n??1? 0 (size (Pos n??1?))))))

(check-synth)
