; 测试 finite 和 cardinality 函数

; 定义一个颜色枚举
(define-sort-enum Color ($red $green $blue $yellow))

; 定义一个方向枚举
(define-sort-enum Direction ($north $south $east $west))

; 定义一个递归的 datatype (应该是无限的)
(declare-datatype List (
  (nil)
  (cons (head Int) (tail List))
))

; 定义一个非递归的 datatype (应该是有限的)
(declare-datatype Point (
  (point (x Bool) (y Bool))
))

; 测试 finite 函数
(echo (finite Color))      ; 应该输出 true
(echo (finite Direction))  ; 应该输出 true
(echo (finite Bool))       ; 应该输出 true
(echo (finite List))       ; 应该输出 false (递归)
(echo (finite Point))      ; 应该输出 true (非递归)
(echo (finite Int))        ; 应该输出 false (无限)
(echo (finite String))     ; 应该输出 false (无限)

; 测试 cardinality 函数
(echo (card Color))      ; 应该输出 4
(echo (card Direction))  ; 应该输出 4
(echo (card Bool))       ; 应该输出 2
(echo (card Point))      ; 应该输出 4 (2×2)

; 测试改进的 ite 函数
(echo (ite true "hello" "world"))  ; 应该输出 "hello"
(echo (ite false "hello" "world")) ; 应该输出 "world"
(echo (ite true 42 100))           ; 应该输出 42
(echo (ite false 42 100))          ; 应该输出 100