use sygus_parser::Sort;
use sygus_parser::{BitVec, Cmd, SyGuSCmd, Term};

pub fn re_union(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("re.union")
        .arg("x", bv_sort.clone())
        .arg("y", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvor(Term::var("x"), Term::var("y")))
        .build()
}

pub fn re_inter(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("re.inter")
        .arg("x", bv_sort.clone())
        .arg("y", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvand(Term::var("x"), Term::var("y")))
        .build()
}

pub fn re_comp(bv_sort: &Sort) -> SyGuSCmd {
    Cmd::define_fun("re.comp")
        .arg("x", bv_sort.clone())
        .ret(bv_sort.clone())
        .body(BitVec::bvnot(Term::var("x")))
        .build()
}
