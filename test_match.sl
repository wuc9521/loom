(define-sort-enum Color ($red $green))

(define-fun test ((c Color)) String
  (match c (
    ($red "red")
    ($green "green")
  ))
)

(echo (test $red))