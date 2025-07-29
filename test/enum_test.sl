; 定义一个颜色枚举
(define-sort-enum Color ($red $green $blue $yellow))

; 定义一个方向枚举
(define-sort-enum Direction ($north $south $east $west))

; 定义一个状态枚举
(define-sort-enum Status ($pending $running $completed $failed))

; 测试颜色相关的函数
(define-fun color-to-string ((c Color)) String
  (match c
    ($red "red")
    ($green "green")
    ($blue "blue")
    ($yellow "yellow")
  )
)

; 测试方向相关的函数
(define-fun is-vertical ((d Direction)) Bool
  (match d
    ($north true)
    ($south true)
    (_ false)
  )
)

; 测试状态相关的函数
(define-fun is-active ((s Status)) Bool
  (match s
    ($pending true)
    ($running true)
    (_ false)
  )
)

; 测试枚举值的使用
(echo (color-to-string $red))
(echo (color-to-string $blue))
(echo (is-vertical $north))
(echo (is-vertical $east))
(echo (is-active $running))
(echo (is-active $completed))