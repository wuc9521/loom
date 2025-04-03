use std::iter::Map;

use super::{grammar::CFG, signature::Sort};

pub struct SynthFun {
    pub args: Map<String, Sort>, // (x Int)
    pub ret_sort: Sort,
    pub cfg: CFG,
}

impl SynthFun {
    pub fn new() -> Self {
        SynthFun {
            args: unimplemented!(),
            ret_sort: Sort {
                name: unimplemented!(),
            },
            cfg: unimplemented!(),
        };
    }
}
