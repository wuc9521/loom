(declare-datatypes ((List 0)) (
  ((nil)
   (cons (head Int) (tail List)))
))

(define-fun debug-cons () List
  (cons 5 nil))

(define-fun test-cons () Int
  (match (cons 5 nil)
    ((nil 0)
     ((cons h t) h))))

(echo "debug-cons result:")
(echo debug-cons)
(echo "test-cons result:")
(echo test-cons)