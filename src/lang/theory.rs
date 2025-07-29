use crate::parser::LSFile;
use crate::theory::Logic;

pub trait SygusDesugar {
    fn desugar(&self, sygus_file: &sygus_parser::SyGuSFile) -> LSFile;
}

impl SygusDesugar for Logic {
    fn desugar(&self, sygus_file: &sygus_parser::SyGuSFile) -> LSFile {
        match self {
            Logic::RE => crate::theory::re::sugar::desugar(sygus_file),
            _ => unimplemented!("desugar not implemented for this logic"),
        }
    }
}