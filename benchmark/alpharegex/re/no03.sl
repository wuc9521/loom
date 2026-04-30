(require "./theories/main.sl")
(require "../common/models/no03.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p0101 0 (size (Pos p0101)))))
(constraint (RegLan.Sem r (ReAsp p_0101_ 0 (size (Pos p_0101_)))))
(constraint (RegLan.Sem r (ReAsp p__0101__ 0 (size (Pos p__0101__)))))
(constraint (not (RegLan.Sem r (ReAsp n0 0 (size (Pos n0))))))
(constraint (not (RegLan.Sem r (ReAsp n1 0 (size (Pos n1))))))
(constraint (not (RegLan.Sem r (ReAsp n?? 0 (size (Pos n??))))))
(constraint (not (RegLan.Sem r (ReAsp n??? 0 (size (Pos n???))))))
(constraint (not (RegLan.Sem r (ReAsp n???0 0 (size (Pos n???0))))))
(constraint (not (RegLan.Sem r (ReAsp n?0?? 0 (size (Pos n?0??))))))
(constraint (not (RegLan.Sem r (ReAsp n1??? 0 (size (Pos n1???))))))
(constraint (not (RegLan.Sem r (ReAsp n??1? 0 (size (Pos n??1?))))))

(check-synth)
