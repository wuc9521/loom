(set-logic ALL)
(require "./theories/main.sl")
(require "../common/models/no06.sl")

(synth-fun g () CFG)

(constraint (CFG.Sem g (CFG.Asp p___ 0 (size (Pos p___)) reset.S)))
(constraint (CFG.Sem g (CFG.Asp p______ 0 (size (Pos p______)) reset.S)))
(constraint (not (CFG.Sem g (CFG.Asp n? 0 (size (Pos n?)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n?? 0 (size (Pos n??)) reset.S))))
(constraint (not (CFG.Sem g (CFG.Asp n???? 0 (size (Pos n????)) reset.S))))

(check-synth)
