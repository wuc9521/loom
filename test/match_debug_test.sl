; 定义一个简单的颜色枚举
(define-sort-enum Color ($red $green $blue))

; 测试枚举值
(echo $red)

; 测试match表达式
(define-fun match-test ((c Color)) String
  (match c
    ($red "red")
    ($green "green")
    ($blue "blue")
  )
)

; 测试函数调用
(echo (match-test $red))