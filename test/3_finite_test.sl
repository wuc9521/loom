; test finiteness typechecking
(define-sort-enum Color ($red $green $blue $yellow))
(define-sort-enum Direction ($north $south $east $west))

(declare-datatype List (
  (nil)
  (cons (head Int) (tail List))
))

(declare-datatype Point (
  (point (x Bool) (y Bool))
))

(echo (finite Color))      ; true
(echo (finite Direction))  ; true
(echo (finite Bool))       ; true
(echo (finite List))       ; false
(echo (finite Point))      ; true
(echo (finite Int))        ; false
(echo (finite String))     ; false

(echo (card Color))      ; 4
(echo (card Direction))  ; 4
(echo (card Bool))       ; 2
(echo (card Point))      ; 4 (2×2)

(echo (ite true "hello" "world"))  ; "hello"
(echo (ite false "hello" "world")) ; "world"
(echo (ite true 42 100))           ; 42
(echo (ite false 42 100))          ; 100