(require "theories/substring-phone.sl")

(define-model ex0
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11))
  (define-univ Chr (c0 c1 c2 c3 c4 c5 c8 c9 cDash))
  (interp-fun in.chr
    (match (
      (($0) c9) (($1) c3) (($2) c8) (($3) cDash)
      (($4) c2) (($5) c4) (($6) c2) (($7) cDash)
      (($8) c5) (($9) c0) (($10) c4)
      ((_) c0)
    )))
  (interp-fun out.chr
    (match (
      (($0) c9) (($1) c3) (($2) c8)
      ((_) c0)
    )))
)

(define-model ex1
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11))
  (define-univ Chr (c0 c1 c3 c4 c5 c6 c8 c9 cDash))
  (interp-fun in.chr
    (match (
      (($0) c3) (($1) c0) (($2) c8) (($3) cDash)
      (($4) c9) (($5) c1) (($6) c6) (($7) cDash)
      (($8) c5) (($9) c4) (($10) c5)
      ((_) c0)
    )))
  (interp-fun out.chr
    (match (
      (($0) c3) (($1) c0) (($2) c8)
      ((_) c0)
    )))
)

(define-model ex2
  (define-univ Pos ($0 $1 $2 $3 $4 $5 $6 $7 $8 $9 $10 $11))
  (define-univ Chr (c0 c2 c3 c4 c5 c6 c7 c9 cDash))
  (interp-fun in.chr
    (match (
      (($0) c6) (($1) c2) (($2) c3) (($3) cDash)
      (($4) c5) (($5) c9) (($6) c9) (($7) cDash)
      (($8) c7) (($9) c4) (($10) c9)
      ((_) c0)
    )))
  (interp-fun out.chr
    (match (
      (($0) c6) (($1) c2) (($2) c3)
      ((_) c0)
    )))
)

(synth-fun f () StrProg)
(constraint (StrProg.OutEq f (ExAsp ex0)))
(constraint (StrProg.OutEq f (ExAsp ex1)))
(constraint (StrProg.OutEq f (ExAsp ex2)))
(check-synth)
