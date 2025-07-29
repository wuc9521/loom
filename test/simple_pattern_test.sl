(declare-datatypes ((List 0)) (
  ((nil)
   (cons (head Int) (tail List)))
))

(define-fun test-nil () Int
  (match nil
    ((nil 42)
     ((cons h t) 0))))

(define-fun test-cons () Int
  (match (cons 5 nil)
    ((nil 0)
     ((cons h t) h))))

(echo "test-nil result:")
(echo test-nil)
(echo "test-cons result:")
(echo test-cons)