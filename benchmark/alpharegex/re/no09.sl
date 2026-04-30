(require "./theories/main.sl")
(require "../common/models/no09.sl")

(synth-fun regex () RegLan)

(constraint (RegLan.Sem r (ReAsp p0____ 0 (size (Pos p0____)))))
(constraint (RegLan.Sem r (ReAsp p_0____ 0 (size (Pos p_0____)))))
(constraint (RegLan.Sem r (ReAsp p__0____ 0 (size (Pos p__0____)))))
(constraint (not (RegLan.Sem r (ReAsp n1???? 0 (size (Pos n1????))))))
(constraint (not (RegLan.Sem r (ReAsp n?1???? 0 (size (Pos n?1????))))))
(constraint (not (RegLan.Sem r (ReAsp n??1???? 0 (size (Pos n??1????))))))

(check-synth)
