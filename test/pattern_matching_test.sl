(declare-datatypes ((List 0)) (
  ((nil)
   (cons (head Int) (tail List)))
))

(define-fun test-pattern-matching ((x List)) Int
  (match x
    ((nil 0)
     ((cons h t) (+ h (test-pattern-matching t))))))

(echo (test-pattern-matching nil))
(echo (test-pattern-matching (cons 1 nil)))
(echo (test-pattern-matching (cons 1 (cons 2 nil))))