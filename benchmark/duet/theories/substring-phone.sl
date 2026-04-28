(provide Pos Chr StrProg ExAsp CharAsp StrProg.Len StrProg.Char StrProg.OutEq in.chr out.chr
slice.0.2 slice.0.3 slice.0.4 slice.1.3)
(set-logic ALL)

(declare-sort Pos 0)
(declare-sort Chr 0)

(declare-fun in.chr (Pos) Chr)
(declare-fun out.chr (Pos) Chr)

(declare-term-types
 ((StrProg 0))
 (((slice.0.2)
   (slice.0.3)
   (slice.0.4)
   (slice.1.3))))

(define-finite ExAsp
  (record (model Model)))

(define-finite CharAsp
  (record (model Model) (k Pos)))

(define-semantics
  ((StrProg.Len (p StrProg) (asp ExAsp)))
  :returns Pos
  ((match p
     ((slice.0.2) $2)
     ((slice.0.3) $3)
     ((slice.0.4) $4)
     ((slice.1.3) $2))))

(define-semantics
  ((StrProg.Char (p StrProg) (asp CharAsp)))
  :returns Chr
  ((match p
     ((slice.0.2)
      (ite (= (k asp) $0) (in.chr $0) (in.chr $1)))
     ((slice.0.3)
      (ite (= (k asp) $0) (in.chr $0)
        (ite (= (k asp) $1) (in.chr $1) (in.chr $2))))
     ((slice.0.4)
      (ite (= (k asp) $0) (in.chr $0)
        (ite (= (k asp) $1) (in.chr $1)
          (ite (= (k asp) $2) (in.chr $2) (in.chr $3)))))
     ((slice.1.3)
      (ite (= (k asp) $0) (in.chr $1) (in.chr $2))))))

(define-semantics
  ((StrProg.OutEq (p StrProg) (asp ExAsp)))
  ((match p
     ((slice.0.2) false)
     ((slice.0.3)
      (and
        (= (in.chr $0) (out.chr $0))
        (= (in.chr $1) (out.chr $1))
        (= (in.chr $2) (out.chr $2))))
     ((slice.0.4) false)
     ((slice.1.3) false))))
