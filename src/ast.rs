pub enum Command {
    SetLogic(String),              // (set-logic FOL)
    DeclareSort { name: String, arity: u32 },  // (declare-sort U 0)
    DefineSignature {
        name: String,
        items: Vec<SignatureItem>, // (declare-fun s () U)之类的
    },
    DefineModel {
        model_name: String,
        signature_name: String,
        items: Vec<ModelItem>,
    },
    DefineGrammar {
        grammar_name: String,
        rules: Vec<GrammarRule>,
    },
    SynthForm {
        func_name: String,
        signature: String,
        grammar: String,
    },
    Constraint(String),            // (constraint (models G1 f))，先简单字符串化
    CheckSynth,                    // (check-synth)
}

#[derive(Debug, Clone)]
pub enum SignatureItem {
    DeclareFun {
        name: String,
        arg_sorts: Vec<String>,
        ret_sort: String,
    },
}

#[derive(Debug, Clone)]
pub enum ModelItem {
    DefineUniverse {
        sort: String,
        elements: Vec<String>,
    },
    Interpret {
        name: String,         
        body: InterpretBody,  
    },
}

#[derive(Debug, Clone)]
pub enum InterpretBody {
    Const(String),  // (interpret s a)
    Match(Vec<(Vec<String>, bool)>), // 简化表示 (match ((a a) True) ((_ _) False) ...)
}

#[derive(Debug, Clone)]
pub enum GrammarRule {
    NonTerminal(String, String),   // (Term U)
    Expansions(String, Vec<String>), // (Term s t x y) or (Fml (and Fml Fml) ...)
}
