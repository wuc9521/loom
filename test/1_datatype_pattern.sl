; 测试 datatype pattern matching

; 定义一个简单的 List datatype
(declare-datatype List (
  (nil)
  (cons (head Int) (tail List))
))

; 定义一个简单的 Tree datatype
(declare-datatype Tree (
  (leaf (value Int))
  (node (left Tree) (right Tree))
))

; 定义一个 Option datatype
(declare-datatype Option (
  (none)
  (some (value Int))
))

; 测试 List 的 pattern matching
(define-fun-rec list-length ((l List)) Int
  (match l
    (
      (nil 0)
      ((cons h t) (+ 1 (list-length t)))
    )
  )
)

; 测试 Tree 的 pattern matching
(define-fun-rec tree-sum ((t Tree)) Int
  (match t (
    ((leaf v) v)
    ((node l r) (+ (tree-sum l) (tree-sum r)))
  ))
)

; 测试 Option 的 pattern matching
(define-fun option-value ((opt Option)) Int
  (match opt (  
    (none 0)
    ((some v) v)
  ))
)

; 创建一些测试数据
(define-fun test-list () List
  (cons 1 (cons 2 (cons 3 nil)))
)

(define-fun test-tree () Tree
  (node (leaf 1) (node (leaf 2) (leaf 3)))
)

(define-fun test-option () Option
  (some 42)
)

; 测试输出
(echo (list-length test-list))
(echo (tree-sum test-tree))
(echo (option-value test-option))
(echo (option-value none))