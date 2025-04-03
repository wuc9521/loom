#[allow(non_camel_case_types)]
pub enum Logic {
    LIA,
    QF_LIA,
    BV,
    Unknown,
}

impl Logic {
    pub fn from_str(s: &str) -> Logic {
        match s {
            "LIA" => Logic::LIA,
            "QF_LIA" => Logic::QF_LIA,
            "BV" => Logic::BV,
            _ => Logic::Unknown,
        }
    }
}
