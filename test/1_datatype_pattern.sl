; (set-logic ALL)

; test datatype pattern matching
(declare-datatype List (
  (nil)
  (cons (head Int) (tail List))
))

(declare-datatype Tree (
  (leaf (value Int))
  (node (left Tree) (right Tree))
))

(declare-datatype Option (
  (none)
  (some (value Int))
))

(define-fun-rec list-length ((l List)) Int
  (match l
    (
      (nil 0)
      ((cons h t) (+ 1 (list-length t)))
    )
  )
)

(define-fun-rec tree-sum ((t Tree)) Int
  (match t (
    ((leaf v) v)
    ((node l r) (+ (tree-sum l) (tree-sum r)))
  ))
)

(define-fun option-value ((opt Option)) Int
  (match opt (  
    (none 0)
    ((some v) v)
  ))
)

(define-fun test-list () List
  (cons 1 (cons 2 (cons 3 nil)))
)

(define-fun test-tree () Tree
  (node (leaf 1) (node (leaf 2) (leaf 3)))
)

(define-fun test-option () Option
  (some 42)
)

(echo (list-length test-list)) ; 3
(echo (tree-sum test-tree)) ; 6
(echo (option-value test-option)) ; 42
(echo (option-value none)) ; 0