(provide FoAsp0 Term.0.abp.index_t Term.0.abp.index_t.Sem Term.0.abp.value Term.0.abp.value.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.abp.index_t fol.0.eq.abp.value fol.0.atom.abp.ack_received fol.0.atom.abp.ack_sent fol.0.atom.abp.data_received fol.0.atom.abp.data_sent fol.0.atom.abp.index.le fol.0.atom.abp.receiver_bit fol.0.atom.abp.receiver_scheduled fol.0.atom.abp.sender_bit fol.0.atom.abp.sender_scheduled fol.0.term.abp.index.zero fol.0.term.abp.receiver_i fol.0.term.abp.sender_gen_i fol.0.term.abp.sender_i fol.0.term.abp.tp1.sk0 fol.0.term.abp.bot fol.0.term.abp.receiver_array fol.0.term.abp.sender_array)
(require "../../common/signatures/alternating_bit_protocol-232f7bb5.sl")

(declare-term-types
 ((Formula0 0) (Term.0.abp.index_t 0) (Term.0.abp.value 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.abp.index_t Term.0.abp.index_t Term.0.abp.index_t)
   (fol.0.eq.abp.value Term.0.abp.value Term.0.abp.value)
   (fol.0.atom.abp.ack_received )
   (fol.0.atom.abp.ack_sent )
   (fol.0.atom.abp.data_received )
   (fol.0.atom.abp.data_sent )
   (fol.0.atom.abp.index.le Term.0.abp.index_t Term.0.abp.index_t)
   (fol.0.atom.abp.receiver_bit )
   (fol.0.atom.abp.receiver_scheduled )
   (fol.0.atom.abp.sender_bit )
   (fol.0.atom.abp.sender_scheduled ))
  ((fol.0.term.abp.index.zero)
   (fol.0.term.abp.receiver_i)
   (fol.0.term.abp.sender_gen_i)
   (fol.0.term.abp.sender_i)
   (fol.0.term.abp.tp1.sk0))
  ((fol.0.term.abp.bot)
   (fol.0.term.abp.receiver_array Term.0.abp.index_t)
   (fol.0.term.abp.sender_array Term.0.abp.index_t))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.abp.index_t.Sem (term Term.0.abp.index_t) (asp FoAsp0)))
  :returns abp.index_t
  ((match term
     ((fol.0.term.abp.index.zero) abp.index.zero)
     ((fol.0.term.abp.receiver_i) abp.receiver_i)
     ((fol.0.term.abp.sender_gen_i) abp.sender_gen_i)
     ((fol.0.term.abp.sender_i) abp.sender_i)
     ((fol.0.term.abp.tp1.sk0) abp.tp1.sk0)
  )))

(define-semantics
  ((Term.0.abp.value.Sem (term Term.0.abp.value) (asp FoAsp0)))
  :returns abp.value
  ((match term
     ((fol.0.term.abp.bot) abp.bot)
     ((fol.0.term.abp.receiver_array a0) (abp.receiver_array (Term.0.abp.index_t.Sem a0 asp)))
     ((fol.0.term.abp.sender_array a0) (abp.sender_array (Term.0.abp.index_t.Sem a0 asp)))
  )))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.eq.abp.index_t lhs rhs)
      (= (Term.0.abp.index_t.Sem lhs asp) (Term.0.abp.index_t.Sem rhs asp)))
     ((fol.0.eq.abp.value lhs rhs)
      (= (Term.0.abp.value.Sem lhs asp) (Term.0.abp.value.Sem rhs asp)))
     ((fol.0.atom.abp.ack_received ) (abp.ack_received))
     ((fol.0.atom.abp.ack_sent ) (abp.ack_sent))
     ((fol.0.atom.abp.data_received ) (abp.data_received))
     ((fol.0.atom.abp.data_sent ) (abp.data_sent))
     ((fol.0.atom.abp.index.le a0 a1) (abp.index.le (Term.0.abp.index_t.Sem a0 asp) (Term.0.abp.index_t.Sem a1 asp)))
     ((fol.0.atom.abp.receiver_bit ) (abp.receiver_bit))
     ((fol.0.atom.abp.receiver_scheduled ) (abp.receiver_scheduled))
     ((fol.0.atom.abp.sender_bit ) (abp.sender_bit))
     ((fol.0.atom.abp.sender_scheduled ) (abp.sender_scheduled))
  )))
