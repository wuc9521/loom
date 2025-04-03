
pub struct Signature {

}

pub enum Declared {
    Func(Vec<Sort>, Sort),
    QVar(Sort),
    Sort(String)
}

pub struct Sort {
    pub name : String, 
}