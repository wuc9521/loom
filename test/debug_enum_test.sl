; 定义一个简单的颜色枚举
(define-sort-enum Color ($red $green $blue))

; 测试枚举值
(echo $red)

; 测试函数调用
(define-fun test-fun ((c Color)) String
  (match c
    ($red "red")
    ($green "green")
    ($blue "blue")
  )
)

; 测试函数调用
(echo (test-fun $red))