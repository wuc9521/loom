use cvc5::{DatatypeConstructorDecl, Kind, Solver, Term, TermManager};
use std::collections::HashMap;
use std::time::Instant;
use sygus_parser::{
    GrammarDef, GroupedRuleList, Identifier, Index, Literal, Sort, SortedVar, SyGuSCmd,
    SyGuSGTerm, SyGuSTerm,
};

pub fn solve(cmds: &[SyGuSCmd], use_fast_enum: bool) -> Result<String, String> {
    let tm = TermManager::new();
    let mut solver = Solver::new(&tm);
    solver.set_option("sygus", "true");
    if use_fast_enum {
        solver.set_option("sygus-enum", "fast");
    }
    let mut bridge = Bridge::new(&tm, &mut solver);
    for cmd in cmds {
        bridge.exec_cmd(cmd)?;
    }
    bridge.finish()
}

struct Bridge<'tm, 'slv> {
    tm: &'tm TermManager,
    solver: &'slv mut Solver<'tm>,
    declared_sorts: HashMap<String, cvc5::Sort<'tm>>,
    funs: HashMap<String, Term<'tm>>,
    synth: Option<SynthInfo<'tm>>,
}

struct SynthInfo<'tm> {
    name: String,
    args: Vec<SortedVar>,
    sort: Sort,
    term: Term<'tm>,
}

impl<'tm, 'slv> Bridge<'tm, 'slv> {
    fn new(tm: &'tm TermManager, solver: &'slv mut Solver<'tm>) -> Self {
        Self {
            tm,
            solver,
            declared_sorts: HashMap::new(),
            funs: HashMap::new(),
            synth: None,
        }
    }

    fn finish(&mut self) -> Result<String, String> {
        let synth = self
            .synth
            .as_ref()
            .ok_or_else(|| "missing synth-fun command".to_string())?;
        let start = Instant::now();
        let result = self.solver.check_synth();
        log::info!("native cvc5 check_synth completed in {:?}", start.elapsed());
        if result.has_no_solution() {
            return Ok("infeasible".to_string());
        }
        if result.is_unknown() || !result.has_solution() {
            return Ok(result.to_string());
        }
        let start = Instant::now();
        let solution = self.solver.get_synth_solution(synth.term.clone());
        log::info!(
            "native cvc5 get_synth_solution completed in {:?}",
            start.elapsed()
        );
        let body = lambda_body(solution);
        Ok(format!(
            "(\n(define-fun {} ({}) {} {})\n)",
            synth.name,
            synth
                .args
                .iter()
                .map(|arg| arg.to_string())
                .collect::<Vec<_>>()
                .join(" "),
            synth.sort,
            body
        ))
    }

    fn exec_cmd(&mut self, cmd: &SyGuSCmd) -> Result<(), String> {
        match cmd {
            SyGuSCmd::SetLogic(logic) => {
                self.solver.set_logic(logic);
            }
            SyGuSCmd::SetOption(key, value) => {
                self.solver
                    .set_option(key.trim_start_matches(':'), &literal_text(value)?);
            }
            SyGuSCmd::DeclareSort(name, arity) => {
                let sort = self.solver.declare_sort(name, *arity as u32);
                self.declared_sorts.insert(name.clone(), sort);
            }
            SyGuSCmd::DeclareDatatype(name, decl) => {
                let ctors = decl
                    .iter()
                    .map(|ctor| self.translate_ctor_decl(ctor))
                    .collect::<Result<Vec<_>, _>>()?;
                let sort = self.solver.declare_dt(name, &ctors);
                self.declared_sorts.insert(name.clone(), sort);
            }
            SyGuSCmd::DefineFun(name, args, sort, body) => {
                let start = Instant::now();
                let bound = args
                    .iter()
                    .map(|arg| {
                        let sort = self.translate_sort(&arg.sort)?;
                        Ok(self.tm.mk_var(sort, &arg.name))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let local = args
                    .iter()
                    .zip(bound.iter())
                    .map(|(arg, term)| (arg.name.clone(), term.clone()))
                    .collect::<HashMap<_, _>>();
                let body = self.translate_term(body, &local)?;
                let ret_sort = self.translate_sort(sort)?;
                let fun = self.solver.define_fun(name, &bound, ret_sort, body, true);
                self.funs.insert(name.clone(), fun);
                log::info!("native cvc5 define-fun {} loaded in {:?}", name, start.elapsed());
            }
            SyGuSCmd::SynthFun(name, args, sort, grammar) => {
                let start = Instant::now();
                let arg_terms = args
                    .iter()
                    .map(|arg| {
                        let sort = self.translate_sort(&arg.sort)?;
                        Ok(self.tm.mk_var(sort, &arg.name))
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                let ret_sort = self.translate_sort(sort)?;
                let term = if let Some(grammar) = grammar {
                    let grammar = self.translate_grammar(grammar, &arg_terms, args)?;
                    self.solver
                        .synth_fun_with_grammar(name, &arg_terms, ret_sort, &grammar)
                } else {
                    self.solver.synth_fun(name, &arg_terms, ret_sort)
                };
                self.synth = Some(SynthInfo {
                    name: name.clone(),
                    args: args.clone(),
                    sort: sort.clone(),
                    term: term.clone(),
                });
                self.funs.insert(name.clone(), term);
                log::info!("native cvc5 synth-fun {} loaded in {:?}", name, start.elapsed());
            }
            SyGuSCmd::Constraint(term) => {
                let start = Instant::now();
                let term = self.translate_term(term, &HashMap::new())?;
                self.solver.add_sygus_constraint(term);
                log::info!("native cvc5 constraint loaded in {:?}", start.elapsed());
            }
            SyGuSCmd::Assume(term) => {
                let term = self.translate_term(term, &HashMap::new())?;
                self.solver.add_sygus_assume(term);
            }
            SyGuSCmd::CheckSynth => {}
            other => {
                return Err(format!("unsupported native cvc5 command: {other}"));
            }
        }
        Ok(())
    }

    fn translate_grammar(
        &mut self,
        grammar: &GrammarDef,
        args_terms: &[Term<'tm>],
        args: &[SortedVar],
    ) -> Result<cvc5::Grammar<'tm>, String> {
        let mut nonterminals = HashMap::new();
        let symbols = grammar
            .grouped_rule_lists
            .iter()
            .map(|group| {
                let sort = self.translate_sort(&group.sort)?;
                let term = self.tm.mk_var(sort, &group.symbol);
                nonterminals.insert(group.symbol.clone(), term.clone());
                Ok(term)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let mut cvc5_grammar = self.solver.mk_grammar(args_terms, &symbols);
        let grammar_args = args
            .iter()
            .zip(args_terms.iter())
            .map(|(arg, term)| (arg.name.clone(), term.clone()))
            .collect::<HashMap<_, _>>();
        for group in &grammar.grouped_rule_lists {
            self.add_group_rules(&mut cvc5_grammar, group, &nonterminals, &grammar_args)?;
        }
        Ok(cvc5_grammar)
    }

    fn add_group_rules(
        &mut self,
        grammar: &mut cvc5::Grammar<'tm>,
        group: &GroupedRuleList,
        nonterminals: &HashMap<String, Term<'tm>>,
        grammar_args: &HashMap<String, Term<'tm>>,
    ) -> Result<(), String> {
        let symbol = nonterminals
            .get(&group.symbol)
            .cloned()
            .ok_or_else(|| format!("missing grammar symbol {}", group.symbol))?;
        let mut local = nonterminals.clone();
        local.extend(grammar_args.clone());
        let explicit_rules = group
            .terms
            .iter()
            .filter_map(|term| match term {
                SyGuSGTerm::SyGuSTerm(term) => Some(term),
                _ => None,
            })
            .map(|term| self.translate_term(term, &local))
            .collect::<Result<Vec<_>, _>>()?;
        if !explicit_rules.is_empty() {
            grammar.add_rules(symbol.clone(), &explicit_rules);
        }
        for term in &group.terms {
            match term {
                SyGuSGTerm::Constant(_) => grammar.add_any_constant(symbol.clone()),
                SyGuSGTerm::Variable(_) => grammar.add_any_variable(symbol.clone()),
                SyGuSGTerm::SyGuSTerm(_) => {}
            }
        }
        Ok(())
    }

    fn translate_term(
        &mut self,
        term: &SyGuSTerm,
        local: &HashMap<String, Term<'tm>>,
    ) -> Result<Term<'tm>, String> {
        match term {
            SyGuSTerm::Identifier(id) => self.lookup_identifier(id, local),
            SyGuSTerm::Literal(lit) => self.translate_literal(lit),
            SyGuSTerm::Application(id, args) => self.translate_app(id, args, local),
            SyGuSTerm::Annotated(term, _) => self.translate_term(term, local),
            SyGuSTerm::Exists(vars, body) => self.translate_quantifier(Kind::Exists, vars, body, local),
            SyGuSTerm::Forall(vars, body) => self.translate_quantifier(Kind::Forall, vars, body, local),
            SyGuSTerm::Let(bindings, body) => {
                let mut scoped = local.clone();
                for binding in bindings {
                    let value = self.translate_term(&binding.term, &scoped)?;
                    scoped.insert(binding.name.clone(), value);
                }
                self.translate_term(body, &scoped)
            }
        }
    }

    fn translate_quantifier(
        &mut self,
        kind: Kind,
        vars: &[SortedVar],
        body: &SyGuSTerm,
        local: &HashMap<String, Term<'tm>>,
    ) -> Result<Term<'tm>, String> {
        let mut scoped = local.clone();
        let var_terms = vars
            .iter()
            .map(|var| {
                let sort = self.translate_sort(&var.sort)?;
                let term = self.tm.mk_var(sort, &var.name);
                scoped.insert(var.name.clone(), term.clone());
                Ok(term)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let body = self.translate_term(body, &scoped)?;
        let bound = self.tm.mk_term(Kind::VariableList, &var_terms);
        Ok(self.tm.mk_term(kind, &[bound, body]))
    }

    fn translate_app(
        &mut self,
        id: &Identifier,
        args: &[SyGuSTerm],
        local: &HashMap<String, Term<'tm>>,
    ) -> Result<Term<'tm>, String> {
        let args = args
            .iter()
            .map(|arg| self.translate_term(arg, local))
            .collect::<Result<Vec<_>, _>>()?;
        match id {
            Identifier::Symbol(name) => {
                if let Some(kind) = builtin_kind(name) {
                    return Ok(self.tm.mk_term(kind, &args));
                }
                if let Some(fun) = local.get(name).or_else(|| self.funs.get(name)) {
                    if args.is_empty() {
                        return Ok(fun.clone());
                    }
                    let mut children = Vec::with_capacity(args.len() + 1);
                    children.push(fun.clone());
                    children.extend(args);
                    return Ok(self.tm.mk_term(Kind::ApplyUf, &children));
                }
                Err(format!("unknown function or variable {name}"))
            }
            Identifier::Indexed(name, indices) if name == "_ extract" => {
                let [Index::Numeral(hi), Index::Numeral(lo)] = indices.as_slice() else {
                    return Err(format!("unsupported extract indices in ({id})"));
                };
                if args.len() != 1 {
                    return Err(format!("extract expects one argument, got {}", args.len()));
                }
                let op = self.tm.mk_op(Kind::BitvectorExtract, &[*hi as u32, *lo as u32]);
                Ok(self.tm.mk_term_from_op(op, &args))
            }
            _ => Err(format!("unsupported indexed operator {id}")),
        }
    }

    fn lookup_identifier(
        &self,
        id: &Identifier,
        local: &HashMap<String, Term<'tm>>,
    ) -> Result<Term<'tm>, String> {
        let Identifier::Symbol(name) = id else {
            return Err(format!("unsupported indexed identifier {id}"));
        };
        local
            .get(name)
            .or_else(|| self.funs.get(name))
            .cloned()
            .ok_or_else(|| format!("unknown identifier {name}"))
    }

    fn translate_literal(&self, lit: &Literal) -> Result<Term<'tm>, String> {
        match lit {
            Literal::Numeral(value) => Ok(self.tm.mk_integer(*value as i64)),
            Literal::Decimal(value) => Ok(self.tm.mk_real_from_str(&value.to_string())),
            Literal::Bool(true) => Ok(self.tm.mk_true()),
            Literal::Bool(false) => Ok(self.tm.mk_false()),
            Literal::HexConst(bits) => {
                let digits = bits.trim_start_matches("#x");
                Ok(self.tm.mk_bv_from_str((digits.len() * 4) as u32, digits, 16))
            }
            Literal::BinConst(bits) => {
                let digits = bits.trim_start_matches("#b");
                Ok(self.tm.mk_bv_from_str(digits.len() as u32, digits, 2))
            }
            Literal::StringConst(text) => {
                let unquoted = text.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(text);
                Ok(self.tm.mk_string(unquoted, true))
            }
        }
    }

    fn translate_sort(&mut self, sort: &Sort) -> Result<cvc5::Sort<'tm>, String> {
        match sort {
            Sort::Simple(Identifier::Symbol(name)) => match name.as_str() {
                "Bool" => Ok(self.tm.boolean_sort()),
                "Int" => Ok(self.tm.integer_sort()),
                "Real" => Ok(self.tm.real_sort()),
                "String" => Ok(self.tm.string_sort()),
                "RegLan" => Ok(self.tm.regexp_sort()),
                _ => self
                    .declared_sorts
                    .get(name)
                    .cloned()
                    .ok_or_else(|| format!("unsupported or undeclared sort {name}")),
            },
            Sort::Parameterized(Identifier::Symbol(name), subsorts) if name == "_ BitVec" => {
                let [Sort::Simple(Identifier::Symbol(width))] = subsorts.as_slice() else {
                    return Err(format!("invalid bit-vector sort {sort}"));
                };
                let width = width
                    .parse::<u32>()
                    .map_err(|err| format!("invalid bit-vector width {width}: {err}"))?;
                Ok(self.tm.mk_bv_sort(width))
            }
            _ => Err(format!("unsupported sort {sort}")),
        }
    }

    fn translate_ctor_decl(
        &mut self,
        ctor: &sygus_parser::DTConsDecl,
    ) -> Result<DatatypeConstructorDecl<'tm>, String> {
        let mut decl = self.tm.mk_dt_cons_decl(&ctor.name);
        for arg in &ctor.args {
            let sort = self.translate_sort(&arg.sort)?;
            decl.add_selector(&arg.name, sort);
        }
        Ok(decl)
    }
}

fn lambda_body(term: Term<'_>) -> Term<'_> {
    if term.kind() == Kind::Lambda && term.num_children() >= 2 {
        term.child(1)
    } else {
        term
    }
}

fn literal_text(lit: &Literal) -> Result<String, String> {
    match lit {
        Literal::Numeral(value) => Ok(value.to_string()),
        Literal::Decimal(value) => Ok(value.to_string()),
        Literal::Bool(value) => Ok(value.to_string()),
        Literal::HexConst(bits) => Ok(format!("#x{}", bits.trim_start_matches("#x"))),
        Literal::BinConst(bits) => Ok(format!("#b{}", bits.trim_start_matches("#b"))),
        Literal::StringConst(text) => Ok(text.clone()),
    }
}

fn builtin_kind(name: &str) -> Option<Kind> {
    Some(match name {
        "not" => Kind::Not,
        "and" => Kind::And,
        "or" => Kind::Or,
        "=" => Kind::Equal,
        "ite" => Kind::Ite,
        "=>" => Kind::Implies,
        "xor" => Kind::Xor,
        "distinct" => Kind::Distinct,
        "+" => Kind::Add,
        "-" => Kind::Sub,
        "*" => Kind::Mult,
        "div" => Kind::IntsDivision,
        "mod" => Kind::IntsModulus,
        "abs" => Kind::Abs,
        "<" => Kind::Lt,
        "<=" => Kind::Leq,
        ">" => Kind::Gt,
        ">=" => Kind::Geq,
        "bvnot" => Kind::BitvectorNot,
        "bvand" => Kind::BitvectorAnd,
        "bvor" => Kind::BitvectorOr,
        "bvxor" => Kind::BitvectorXor,
        "bvadd" => Kind::BitvectorAdd,
        "bvsub" => Kind::BitvectorSub,
        "bvmul" => Kind::BitvectorMult,
        "bvudiv" => Kind::BitvectorUdiv,
        "bvurem" => Kind::BitvectorUrem,
        "bvsdiv" => Kind::BitvectorSdiv,
        "bvsrem" => Kind::BitvectorSrem,
        "bvshl" => Kind::BitvectorShl,
        "bvlshr" => Kind::BitvectorLshr,
        "bvashr" => Kind::BitvectorAshr,
        "bvult" => Kind::BitvectorUlt,
        "bvule" => Kind::BitvectorUle,
        "bvugt" => Kind::BitvectorUgt,
        "bvuge" => Kind::BitvectorUge,
        "bvslt" => Kind::BitvectorSlt,
        "bvsle" => Kind::BitvectorSle,
        "bvsgt" => Kind::BitvectorSgt,
        "bvsge" => Kind::BitvectorSge,
        "concat" => Kind::BitvectorConcat,
        _ => return None,
    })
}
