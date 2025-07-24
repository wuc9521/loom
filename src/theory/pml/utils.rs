use sygus_parser::Sort;
use sygus_parser::{
    api::{BitVec, Cmd, Term},
    SyGuSCmd,
};

pub fn modal_and(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("modal.and")
        .arg("x", bv_sort.clone())
        .arg("y", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvand(Term::var("x"), Term::var("y")))
        .build()
}

pub fn modal_or(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("modal.or")
        .arg("x", bv_sort.clone())
        .arg("y", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvor(Term::var("x"), Term::var("y")))
        .build()
}

pub fn modal_not(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("modal.not")
        .arg("x", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvnot(Term::var("x")))
        .build()
}
