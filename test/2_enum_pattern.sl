; test enum-sort pattern matching
(define-sort-enum Color ($red $green $blue $yellow))
(define-sort-enum Direction ($north $south $east $west))
(define-sort-enum Status ($pending $running $completed $failed))
(define-fun color-to-string ((c Color)) String
(match c (
  ($red "red")
  ($green "green")
  ($blue "blue")
  ($yellow "yellow")
))
)

(define-fun opposite-direction ((d Direction)) Direction
(match d (
  ($north $south)
  ($south $north)
  ($east $west)
  ($west $east)
))
)

(define-fun is-final-status ((s Status)) Bool
(match s (
  ($pending false)
  ($running false)
  ($completed true)
  ($failed true)
))
)

(echo (color-to-string $red)) ; "red"
(echo (color-to-string $green)) ; "green"
(echo (opposite-direction $north)) ; $south
(echo (opposite-direction $east)) ; $west
(echo (is-final-status $completed)) ; true
(echo (is-final-status $pending)) ; false