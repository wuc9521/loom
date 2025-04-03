#[allow(non_camel_case_types)]
pub enum Logic {
    LIA,
    QF_LIA,
    BV, 
    Unknown
}
impl Logic {
    pub fn from_str(s: &str) -> Option<Logic> {
        match s {
            "LIA" => Some(Logic::LIA),
            "QF_LIA" => Some(Logic::QF_LIA),
            "BV" => Some(Logic::BV),
            _ => None,
        }
    }
}