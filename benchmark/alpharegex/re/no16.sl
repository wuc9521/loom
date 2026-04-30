(require "./theories/main.sl")
(require "../common/models/no16.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p_ 0 (size (Pos p_)))))
(constraint (RegLan.Sem r (ReAsp p_0 0 (size (Pos p_0)))))
(constraint (RegLan.Sem r (ReAsp p11 0 (size (Pos p11)))))
(constraint (RegLan.Sem r (ReAsp p__0 0 (size (Pos p__0)))))
(constraint (RegLan.Sem r (ReAsp p_11 0 (size (Pos p_11)))))
(constraint (RegLan.Sem r (ReAsp p___0 0 (size (Pos p___0)))))
(constraint (RegLan.Sem r (ReAsp p__11 0 (size (Pos p__11)))))
(constraint (RegLan.Sem r (ReAsp p___11 0 (size (Pos p___11)))))
(constraint (not (RegLan.Sem r (ReAsp n01 0 (size (Pos n01))))))
(constraint (not (RegLan.Sem r (ReAsp n?01 0 (size (Pos n?01))))))
(constraint (not (RegLan.Sem r (ReAsp n??01 0 (size (Pos n??01))))))

(check-synth)
