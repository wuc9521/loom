; 定义一个简单的颜色枚举
(define-sort-enum Color ($red $green $blue))

; 测试枚举值
(echo $red)

; 测试简单的函数定义
(define-fun simple-test ((c Color)) String
  "test"
)

; 测试函数调用
(echo (simple-test $red))