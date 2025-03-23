#[derive(Debug)]
pub struct Program {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum Stmt {
    SetLogic {
        logic: String,
    },
    DeclareSort {
        name: String,
        arity: i32,
    },
    DefineSignature {
        name: String,
        signature: Signature,
    },
    DefineModel {
        model_name: String,
        signature_name: String,
        universe: UniverseDef,
        interps: Interpretations,
    },
    DefineGrammar {
        name: String,
        grammar: Grammar,
    },
    SynthForm {
        name: String,
        signature_name: String,
        grammar_name: String,
    },
    Constraint {
        formula: (),
    }, // Placeholder for formula representation
    CheckSynth,
}

#[derive(Debug)]
pub struct Signature {
    pub statements: Vec<SignatureStmt>,
}

#[derive(Debug)]
pub enum SignatureStmt {
    DeclareFun {
        name: String,
        sorts: Vec<String>,
        sort: String,
    },
}

#[derive(Debug)]
pub struct Interpretations {
    pub statements: Vec<InterpretationStmt>,
}

#[derive(Debug)]
pub enum InterpretationStmt {
    DefineUniverse {
        definition: UniverseDef,
    },
    InterpretGroup {
        bindings: Vec<(String, InterpretationValue)>,
    },
}
#[derive(Debug)]
pub struct UniverseDef {
    pub sort: String,
    pub elements: Vec<String>,
}

#[derive(Debug)]
pub enum InterpretationValue {
    Ident(String),
    Match(Match),
}

#[derive(Debug)]
pub struct Match {
    pub cases: Vec<MatchCase>,
}

#[derive(Debug)]
pub struct MatchCase {
    pub terms: Vec<String>,
    pub bool_value: bool,
}

#[derive(Debug)]
pub struct Grammar {
    pub rules: Vec<GrammarRule>,
}

#[derive(Debug)]
pub enum GrammarRule {
    Terminal {
        name: String,
        sort: String,
    },
    Production {
        name: String,
        production: Vec<String>,
    },
}
