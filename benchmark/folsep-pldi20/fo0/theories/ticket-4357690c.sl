(provide FoAsp0 Term.0.ticket_protocol.thread Term.0.ticket_protocol.thread.Sem Term.0.ticket_protocol.ticket Term.0.ticket_protocol.ticket.Sem Formula0 FO.Sem0 fol.0.true fol.0.false fol.0.not fol.0.and fol.0.or fol.0.eq.ticket_protocol.thread fol.0.eq.ticket_protocol.ticket fol.0.atom.ticket_protocol.le fol.0.atom.ticket_protocol.m fol.0.atom.ticket_protocol.pc1 fol.0.atom.ticket_protocol.pc2 fol.0.atom.ticket_protocol.pc3 fol.0.atom.ticket_protocol.scheduled fol.0.term.ticket_protocol.tp1.sk0 fol.0.term.ticket_protocol.next_ticket fol.0.term.ticket_protocol.service fol.0.term.ticket_protocol.zero)
(require "../../common/signatures/ticket-4357690c.sl")

(declare-term-types
 ((Formula0 0) (Term.0.ticket_protocol.thread 0) (Term.0.ticket_protocol.ticket 0))
 (((fol.0.true)
   (fol.0.false)
   (fol.0.not Formula0)
   (fol.0.and Formula0 Formula0)
   (fol.0.or Formula0 Formula0)
   (fol.0.eq.ticket_protocol.thread Term.0.ticket_protocol.thread Term.0.ticket_protocol.thread)
   (fol.0.eq.ticket_protocol.ticket Term.0.ticket_protocol.ticket Term.0.ticket_protocol.ticket)
   (fol.0.atom.ticket_protocol.le Term.0.ticket_protocol.ticket Term.0.ticket_protocol.ticket)
   (fol.0.atom.ticket_protocol.m Term.0.ticket_protocol.thread Term.0.ticket_protocol.ticket)
   (fol.0.atom.ticket_protocol.pc1 Term.0.ticket_protocol.thread)
   (fol.0.atom.ticket_protocol.pc2 Term.0.ticket_protocol.thread)
   (fol.0.atom.ticket_protocol.pc3 Term.0.ticket_protocol.thread)
   (fol.0.atom.ticket_protocol.scheduled Term.0.ticket_protocol.thread))
  ((fol.0.term.ticket_protocol.tp1.sk0))
  ((fol.0.term.ticket_protocol.next_ticket)
   (fol.0.term.ticket_protocol.service)
   (fol.0.term.ticket_protocol.zero))))

(define-finite FoAsp0 (record (model Model)))

(define-semantics
  ((Term.0.ticket_protocol.thread.Sem (term Term.0.ticket_protocol.thread) (asp FoAsp0)))
  :returns ticket_protocol.thread
  ((match term
     ((fol.0.term.ticket_protocol.tp1.sk0) ticket_protocol.tp1.sk0)
  )))

(define-semantics
  ((Term.0.ticket_protocol.ticket.Sem (term Term.0.ticket_protocol.ticket) (asp FoAsp0)))
  :returns ticket_protocol.ticket
  ((match term
     ((fol.0.term.ticket_protocol.next_ticket) ticket_protocol.next_ticket)
     ((fol.0.term.ticket_protocol.service) ticket_protocol.service)
     ((fol.0.term.ticket_protocol.zero) ticket_protocol.zero)
  )))

(define-semantics
  ((FO.Sem0 (phi Formula0) (asp FoAsp0)))
  ((match phi
     ((fol.0.true) true)
     ((fol.0.false) false)
     ((fol.0.not p) (not (FO.Sem0 p asp)))
     ((fol.0.and p q) (and (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.or p q) (or (FO.Sem0 p asp) (FO.Sem0 q asp)))
     ((fol.0.eq.ticket_protocol.thread lhs rhs)
      (= (Term.0.ticket_protocol.thread.Sem lhs asp) (Term.0.ticket_protocol.thread.Sem rhs asp)))
     ((fol.0.eq.ticket_protocol.ticket lhs rhs)
      (= (Term.0.ticket_protocol.ticket.Sem lhs asp) (Term.0.ticket_protocol.ticket.Sem rhs asp)))
     ((fol.0.atom.ticket_protocol.le a0 a1) (ticket_protocol.le (Term.0.ticket_protocol.ticket.Sem a0 asp) (Term.0.ticket_protocol.ticket.Sem a1 asp)))
     ((fol.0.atom.ticket_protocol.m a0 a1) (ticket_protocol.m (Term.0.ticket_protocol.thread.Sem a0 asp) (Term.0.ticket_protocol.ticket.Sem a1 asp)))
     ((fol.0.atom.ticket_protocol.pc1 a0) (ticket_protocol.pc1 (Term.0.ticket_protocol.thread.Sem a0 asp)))
     ((fol.0.atom.ticket_protocol.pc2 a0) (ticket_protocol.pc2 (Term.0.ticket_protocol.thread.Sem a0 asp)))
     ((fol.0.atom.ticket_protocol.pc3 a0) (ticket_protocol.pc3 (Term.0.ticket_protocol.thread.Sem a0 asp)))
     ((fol.0.atom.ticket_protocol.scheduled a0) (ticket_protocol.scheduled (Term.0.ticket_protocol.thread.Sem a0 asp)))
  )))
