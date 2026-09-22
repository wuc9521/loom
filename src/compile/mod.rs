use crate::backends::smt2::ast::HornProg;
use crate::fxmap::HashMap;
use crate::parser::LoomProg;
use std::cell::RefCell;
use std::time::Instant;
use sygus_parser::{Identifier, Sort, SyGuSCmd, SyGuSTerm};

mod aspects;
mod bexpr;
mod bisim;
mod chc;
mod closure;
mod compiler;
mod context;
mod eval;
mod eval_ctx;
mod expr;
pub mod optim;
mod parent;
mod semantics;
mod syntax;
pub mod types;
mod verifier;

use context::CompileContext;
pub use syntax::{TermConstructor, TermSignature};
pub use types::AspectKey;

#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub optimize: bool,
    pub bisim: bool,
}

impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            optimize: true,
            bisim: true,
        }
    }
}

thread_local! {
    static COMPILE_OPTS: RefCell<CompileOptions> = RefCell::new(CompileOptions::default());
}

pub fn init_options(opts: CompileOptions) {
    COMPILE_OPTS.with(|cell| *cell.borrow_mut() = opts);
}

pub fn opts() -> CompileOptions {
    COMPILE_OPTS.with(|cell| cell.borrow().clone())
}

pub fn failing_constraint_indices(cmds: &[SyGuSCmd], solution: &str) -> Result<Vec<usize>, String> {
    let verifier = verifier::Verifier::new(cmds)?;
    verifier.failing_constraint_indices(solution)
}

impl LoomProg {
    pub fn max_bv_width(&self) -> Result<usize, String> {
        let ctx = CompileContext::new(self)?;
        if ctx.is_parent_free() {
            return Ok(ctx
                .sig
                .sort_order()
                .iter()
                .map(|sort| {
                    ctx.semantic_program
                        .domains
                        .iter()
                        .filter(|domain| domain.term_sort == *sort)
                        .map(|domain| domain.width())
                        .sum::<usize>()
                        .max(1)
                })
                .max()
                .unwrap_or(1));
        }
        let mut horn = ctx.compile_chc()?;
        if std::env::var("LOOM_MIN_STATES").map_or(true, |v| v != "0") {
            horn.quotient_bisim();
        }
        Ok(parent::parent_max_bv_width(
            &horn,
            parent::QueryKind::Refuter,
        ))
    }

    pub fn bv_sz(&self) -> Result<usize, String> {
        let ctx = CompileContext::new(self)?;
        let width = ctx
            .semantic_program
            .domains
            .iter()
            .filter(|domain| domain.term_sort == ctx.sig.start_sort())
            .map(|domain| domain.width())
            .sum::<usize>();
        if width == 0 {
            return Err("SyGuS compilation requires at least one semantic state".to_string());
        }
        Ok(width)
    }

    pub fn to_sygus(&self) -> Result<Vec<SyGuSCmd>, String> {
        let semantic_start = Instant::now();
        let ctx = CompileContext::new(self)?;
        log::info!("semantic program built in {:?}", semantic_start.elapsed());
        let encode_start = Instant::now();
        let cmds = ctx.compile()?;
        log::info!("encoded to SyGuS BV in {:?}", encode_start.elapsed());
        self.with_axiom_assume_cmds(cmds)
    }

    pub fn to_chc_ast(&self) -> Result<HornProg, String> {
        CompileContext::new(self)?.compile_chc()
    }

    pub fn to_sygus_bytes(&self) -> Result<Vec<u8>, String> {
        let cmds = self.to_sygus()?;
        let mut out = Vec::new();
        sygus_parser::writer::write_cmds(&mut out, &cmds).map_err(|e| e.to_string())?;
        use std::io::Write as _;
        writeln!(&mut out).map_err(|e| e.to_string())?;
        Ok(out)
    }

    /// Counterexample (refuter) encoding for a parent-bearing program. For a fixed
    /// skeleton, SAT means the skeleton violates some root demand; UNSAT means no
    /// violation certificate exists.
    pub fn parent_refuter_sygus(&self) -> Result<(String, String, String), String> {
        let ctx = CompileContext::new(self)?;
        let synth_name = ctx.synth_name.clone();
        let mut horn = ctx.compile_chc()?;
        // Sound bisimulation quotient: merge predicate states with identical
        // least-model behaviour before encoding (set LOOM_MIN_STATES=0 to skip,
        // for A/B verdict checks).
        if std::env::var("LOOM_MIN_STATES").map_or(true, |v| v != "0") {
            let before = horn.live_predicates().len();
            let after = horn.quotient_bisim();
            log::debug!("state quotient: {before} -> {after} states");
        }
        let (sygus, sort) = parent::parent_sygus(&horn, &synth_name, parent::QueryKind::Refuter);
        let sygus = self.with_axiom_assumes(sygus)?;
        Ok((sygus, sort, synth_name))
    }

    /// String twin of `axiom_assume_cmds` for the parent-refuter dump: same
    /// declare-var + renamed-assume block, spliced before `(check-synth)`.
    fn with_axiom_assumes(&self, sygus: String) -> Result<String, String> {
        let axioms = self.axioms();
        if axioms.is_empty() {
            return Ok(sygus);
        }
        // ponytail: reparse for signatures; main.rs parses the same string again.
        let file = sygus_parser::SyGuSFile::from_str(&sygus)
            .map_err(|e| format!("reparse of generated parent dump failed: {e}"))?;
        let sigs = axiom_sigs(&file.cmds);
        let (declared, assumes) = axiom_decls_and_assumes(axioms, &sigs)?;
        let mut block = String::new();
        for (name, sort) in &declared {
            block.push_str(&format!("(declare-var {name} {sort})\n"));
        }
        for term in &assumes {
            block.push_str(&format!("(assume {term})\n"));
        }
        Ok(sygus.replace("(check-synth)\n", &format!("{block}(check-synth)\n")))
    }

    fn with_axiom_assume_cmds(&self, cmds: Vec<SyGuSCmd>) -> Result<Vec<SyGuSCmd>, String> {
        axiom_assume_cmds(self.axioms(), cmds)
    }

    pub fn parent_eval_bench(&self, max_size: usize) -> Result<(), String> {
        let ctx = CompileContext::new(self)?;
        let horn = ctx.compile_chc()?;
        parent::parent_eval_bench(&horn, max_size);
        Ok(())
    }
}

/// Decode a SyGuS solver's solution for the parent encoding into a plain loom
/// term (the synth-fun named `synth_name`, with annotation args stripped).
pub fn decode_parent_term(stdout: &str, synth_name: &str) -> Option<String> {
    parent::decode_term(stdout, synth_name)
}

/// SyGuS 2.1: an `(assume t)` must be well-sorted, and an axiom law's free
/// variables need `(declare-var x Sort)`. Sorts come from the dump's own
/// operator signatures; a name reused under a different sort by a later law
/// is alpha-renamed — each law is its own ∀-closure, and the backend's law
/// matchers only compare names within one law.
fn axiom_assume_cmds(
    axioms: Vec<SyGuSTerm>,
    mut cmds: Vec<SyGuSCmd>,
) -> Result<Vec<SyGuSCmd>, String> {
    if axioms.is_empty() {
        return Ok(cmds);
    }
    let sigs = axiom_sigs(&cmds);
    let (declared, assumes) = axiom_decls_and_assumes(axioms, &sigs)?;
    let insert_at = cmds
        .iter()
        .position(|cmd| matches!(cmd, SyGuSCmd::CheckSynth))
        .unwrap_or(cmds.len());
    let decls = declared
        .into_iter()
        .map(|(name, sort)| SyGuSCmd::DeclareVar(name, sort));
    let assumes = assumes.into_iter().map(SyGuSCmd::Assume);
    cmds.splice(insert_at..insert_at, decls.chain(assumes));
    Ok(cmds)
}

fn axiom_sigs(cmds: &[SyGuSCmd]) -> HashMap<String, Vec<Sort>> {
    let mut sigs: HashMap<String, Vec<Sort>> = HashMap::default();
    for cmd in cmds {
        let (name, args) = match cmd {
            SyGuSCmd::DefineFun(name, args, _, _)
            | SyGuSCmd::DefineFunRec(name, args, _, _)
            | SyGuSCmd::SynthFun(name, args, _, _) => (name, args),
            _ => continue,
        };
        sigs.entry(name.clone())
            .or_insert_with(|| args.iter().map(|v| v.sort.clone()).collect());
    }
    sigs
}

/// Per axiom: infer free-variable sorts, alpha-rename sort collisions with
/// earlier laws. Returns the declare-var list and the renamed assume terms.
fn axiom_decls_and_assumes(
    axioms: Vec<SyGuSTerm>,
    sigs: &HashMap<String, Vec<Sort>>,
) -> Result<(Vec<(String, Sort)>, Vec<SyGuSTerm>), String> {
    let mut declared: Vec<(String, Sort)> = Vec::new();
    let mut assumes = Vec::with_capacity(axioms.len());
    for axiom in axioms {
        let mut vars: Vec<(String, Sort)> = Vec::new();
        let mut unresolved: Vec<String> = Vec::new();
        axiom_var_sorts(&axiom, &sigs, &mut vars, &mut unresolved)?;
        for name in &unresolved {
            if !vars.iter().any(|(n, _)| n == name) {
                return Err(format!(
                    "cannot infer sort for axiom variable `{name}` in `{axiom}`: it never occurs under a declared operator"
                ));
            }
        }
        let mut renames: HashMap<String, String> = HashMap::default();
        for (name, sort) in &vars {
            match declared.iter().find(|(n, _)| n == name) {
                Some((_, prev)) if prev == sort => {}
                Some(_) => {
                    let fresh = (2..)
                        .map(|i| format!("{name}{i}"))
                        .find(|cand| {
                            !sigs.contains_key(cand) && !declared.iter().any(|(n, _)| n == cand)
                        })
                        .expect("candidate names are unbounded");
                    declared.push((fresh.clone(), sort.clone()));
                    renames.insert(name.clone(), fresh);
                }
                None => declared.push((name.clone(), sort.clone())),
            }
        }
        let axiom = if renames.is_empty() {
            axiom
        } else {
            rename_axiom_vars(axiom, &renames)
        };
        assumes.push(axiom);
    }
    Ok((declared, assumes))
}

/// Collect (name, sort) for the axiom variables under declared operators.
/// Names in untyped positions (the right-hand `x` of `(= (re.union x x) x)`)
/// go to `unresolved` for the caller to match against the collected sorts.
fn axiom_var_sorts(
    term: &SyGuSTerm,
    sigs: &HashMap<String, Vec<Sort>>,
    vars: &mut Vec<(String, Sort)>,
    unresolved: &mut Vec<String>,
) -> Result<(), String> {
    if let SyGuSTerm::Annotated(inner, _) = term {
        return axiom_var_sorts(inner, sigs, vars, unresolved);
    }
    let SyGuSTerm::Application(head, args) = term else {
        return Ok(());
    };
    let sig = sigs.get(head.get_name()).map(Vec::as_slice).unwrap_or(&[]);
    for (i, arg) in args.iter().enumerate() {
        let SyGuSTerm::Identifier(Identifier::Symbol(name)) = arg else {
            axiom_var_sorts(arg, sigs, vars, unresolved)?;
            continue;
        };
        if sigs.contains_key(name) {
            continue; // declared operator or constant
        }
        let Some(sort) = sig.get(i) else {
            unresolved.push(name.clone());
            continue;
        };
        match vars.iter().find(|(n, _)| n == name) {
            Some((_, prev)) if prev != sort => {
                return Err(format!(
                    "axiom variable `{name}` occurs at conflicting sorts `{prev}` and `{sort}` in `{term}`"
                ))
            }
            Some(_) => {}
            None => vars.push((name.clone(), sort.clone())),
        }
    }
    Ok(())
}

fn rename_axiom_vars(term: SyGuSTerm, renames: &HashMap<String, String>) -> SyGuSTerm {
    match term {
        SyGuSTerm::Identifier(Identifier::Symbol(name)) => match renames.get(&name) {
            Some(fresh) => SyGuSTerm::Identifier(Identifier::Symbol(fresh.clone())),
            None => SyGuSTerm::Identifier(Identifier::Symbol(name)),
        },
        SyGuSTerm::Application(head, args) => SyGuSTerm::Application(
            head,
            args.into_iter()
                .map(|arg| rename_axiom_vars(arg, renames))
                .collect(),
        ),
        SyGuSTerm::Annotated(inner, attrs) => {
            SyGuSTerm::Annotated(Box::new(rename_axiom_vars(*inner, renames)), attrs)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests;
