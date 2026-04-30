(require "./theories/main.sl")
(require "../common/models/no14.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p0 0 (size (Pos p0)))))
(constraint (RegLan.Sem r (ReAsp p0__ 0 (size (Pos p0__)))))
(constraint (RegLan.Sem r (ReAsp p0____ 0 (size (Pos p0____)))))
(constraint (RegLan.Sem r (ReAsp p1_ 0 (size (Pos p1_)))))
(constraint (RegLan.Sem r (ReAsp p1___ 0 (size (Pos p1___)))))
(constraint (not (RegLan.Sem r (ReAsp n0? 0 (size (Pos n0?))))))
(constraint (not (RegLan.Sem r (ReAsp n0??? 0 (size (Pos n0???))))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n1?? 0 (size (Pos n1??))))))
(constraint (not (RegLan.Sem r (ReAsp n1???? 0 (size (Pos n1????))))))

(check-synth)
