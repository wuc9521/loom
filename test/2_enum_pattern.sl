; 测试 enum-sort pattern matching

; 定义一个颜色枚举
(define-sort-enum Color ($red $green $blue $yellow))

; 定义一个方向枚举
(define-sort-enum Direction ($north $south $east $west))

; 定义一个状态枚举
(define-sort-enum Status ($pending $running $completed $failed))

; 测试颜色相关的函数
(define-fun color-to-string ((c Color)) String
(match c (
  ($red "red")
  ($green "green")
  ($blue "blue")
  ($yellow "yellow")
))
)

; 测试方向相关的函数
(define-fun opposite-direction ((d Direction)) Direction
(match d (
  ($north $south)
  ($south $north)
  ($east $west)
  ($west $east)
))
)

; 测试状态相关的函数
(define-fun is-final-status ((s Status)) Bool
(match s (
  ($pending false)
  ($running false)
  ($completed true)
  ($failed true)
))
)

; 测试输出
(echo (color-to-string $red))
(echo (color-to-string $green))
(echo (opposite-direction $north))
(echo (opposite-direction $east))
(echo (is-final-status $completed))
(echo (is-final-status $pending))