use crate::parser::ast::{parse, DelisCmd};
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
};
use sygus_parser::SyGuSCmd;

#[derive(Debug)]
struct ModuleForm {
    text: String,
    kind: ModuleFormKind,
}

#[derive(Debug, PartialEq, Eq)]
enum ModuleFormKind {
    Require(Vec<String>),
    Provide(Vec<String>),
    Other,
}

#[derive(Debug)]
struct ModuleSource {
    requires: Vec<String>,
    provides: Vec<String>,
    body: String,
}

#[derive(Default)]
struct ModuleLoader {
    loaded: HashSet<PathBuf>,
    loading: Vec<PathBuf>,
}

impl ModuleLoader {
    fn load(&mut self, path: &Path) -> Result<String, io::Error> {
        let path = canonicalize_existing(path)?;
        if self.loaded.contains(&path) {
            return Ok(String::new());
        }
        if let Some(pos) = self.loading.iter().position(|p| p == &path) {
            let mut cycle = self.loading[pos..]
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>();
            cycle.push(path.display().to_string());
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("module require cycle: {}", cycle.join(" -> ")),
            ));
        }

        self.loading.push(path.clone());
        let raw = fs::read_to_string(&path)?;
        let source = split_module_source(&raw).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{}: {e}", path.display()),
            )
        })?;

        validate_provides(&path, &source)?;

        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let mut expanded = String::new();
        for req in &source.requires {
            let req_path = base.join(req);
            expanded.push_str(&self.load(&req_path)?);
            if !expanded.ends_with('\n') {
                expanded.push('\n');
            }
        }
        expanded.push_str(&source.body);
        if !expanded.ends_with('\n') {
            expanded.push('\n');
        }

        self.loading.pop();
        self.loaded.insert(path);
        Ok(expanded)
    }
}

pub fn load_module_file(path: impl AsRef<Path>) -> Result<String, io::Error> {
    ModuleLoader::default().load(path.as_ref())
}

fn canonicalize_existing(path: &Path) -> Result<PathBuf, io::Error> {
    path.canonicalize().map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("failed to resolve module path {}: {e}", path.display()),
        )
    })
}

fn split_module_source(input: &str) -> Result<ModuleSource, String> {
    let forms = top_level_forms(input)?;
    let mut requires = Vec::new();
    let mut provides = Vec::new();
    let mut body = Vec::new();

    for form in forms {
        match form.kind {
            ModuleFormKind::Require(paths) => requires.extend(paths),
            ModuleFormKind::Provide(names) => provides.extend(names),
            ModuleFormKind::Other => body.push(form.text),
        }
    }

    Ok(ModuleSource {
        requires,
        provides,
        body: body.join("\n"),
    })
}

fn validate_provides(path: &Path, source: &ModuleSource) -> Result<(), io::Error> {
    if source.provides.is_empty() {
        return Ok(());
    }
    let prog = parse(&source.body, None).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "failed to parse module {} while checking provide: {e}",
                path.display()
            ),
        )
    })?;
    let defined = defined_names(&prog.cmds);
    let missing = source
        .provides
        .iter()
        .filter(|name| !defined.contains(*name))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} provides undefined name(s): {}",
                path.display(),
                missing.join(", ")
            ),
        ))
    }
}

fn defined_names(cmds: &[DelisCmd]) -> HashSet<String> {
    let mut names = HashSet::new();
    for cmd in cmds {
        match cmd {
            DelisCmd::As(SyGuSCmd::DeclareSort(name, _)) => {
                names.insert(name.clone());
            }
            DelisCmd::As(SyGuSCmd::SynthFun(name, ..)) => {
                names.insert(name.clone());
            }
            DelisCmd::As(SyGuSCmd::DefineFun(name, ..)) => {
                names.insert(name.clone());
            }
            DelisCmd::TermTypes(types) => {
                for sort in &types.sort_decls {
                    names.insert(sort.symbol.clone());
                }
                for decl in &types.term_type_decls {
                    for cons in &decl.constructors {
                        names.insert(cons.symbol.to_string());
                    }
                }
            }
            DelisCmd::FiniteAspect(aspect) => {
                names.insert(aspect.name.clone());
            }
            DelisCmd::Semantics(semantics) => {
                if let Some(name) = semantics_name(Some(&semantics.header)) {
                    names.insert(name);
                }
            }
            DelisCmd::DeclareFunc(sig) => {
                names.insert(sig.name.clone());
            }
            DelisCmd::DefineModel(model) => {
                names.insert(model.name.clone());
            }
            _ => {}
        }
    }
    names
}

fn semantics_name(header: Option<&String>) -> Option<String> {
    let value = lexpr::from_str(header?).ok()?;
    let outer = value.to_ref_vec()?;
    let signature = if outer.len() == 1 { outer[0] } else { &value };
    let items = signature.to_ref_vec()?;
    Some(items.first()?.as_symbol()?.to_string())
}

fn top_level_forms(input: &str) -> Result<Vec<ModuleForm>, String> {
    let bytes = input.as_bytes();
    let mut forms = Vec::new();
    let mut depth = 0usize;
    let mut start = None;
    let mut in_string = false;
    let mut escaped = false;
    let mut in_comment = false;

    for (idx, byte) in bytes.iter().enumerate() {
        let ch = *byte as char;
        if in_comment {
            if ch == '\n' {
                in_comment = false;
            }
            continue;
        }
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }
        match ch {
            ';' => in_comment = true,
            '"' => in_string = true,
            '(' => {
                if depth == 0 {
                    start = Some(idx);
                }
                depth += 1;
            }
            ')' => {
                if depth == 0 {
                    return Err("unmatched ')' in module source".to_string());
                }
                depth -= 1;
                if depth == 0 {
                    let start = start
                        .take()
                        .ok_or_else(|| "internal module scanner error".to_string())?;
                    let text = input[start..=idx].to_string();
                    let kind = classify_form(&text)?;
                    forms.push(ModuleForm { text, kind });
                }
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err("unclosed top-level form in module source".to_string());
    }
    if in_string {
        return Err("unclosed string literal in module source".to_string());
    }
    Ok(forms)
}

fn classify_form(form: &str) -> Result<ModuleFormKind, String> {
    let tokens = shallow_tokens(form)?;
    match tokens.first().map(String::as_str) {
        Some("require") => {
            let paths = tokens[1..]
                .iter()
                .map(|token| parse_string_token(token, "require"))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ModuleFormKind::Require(paths))
        }
        Some("provide") => Ok(ModuleFormKind::Provide(tokens[1..].to_vec())),
        _ => Ok(ModuleFormKind::Other),
    }
}

fn shallow_tokens(form: &str) -> Result<Vec<String>, String> {
    let inner = form
        .strip_prefix('(')
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("expected parenthesized form, got {form}"))?;
    let mut tokens = Vec::new();
    let bytes = inner.as_bytes();
    let mut idx = 0usize;
    while idx < bytes.len() {
        while idx < bytes.len() && (bytes[idx] as char).is_whitespace() {
            idx += 1;
        }
        if idx == bytes.len() {
            break;
        }
        let start = idx;
        if bytes[idx] as char == '"' {
            idx += 1;
            let mut escaped = false;
            while idx < bytes.len() {
                let ch = bytes[idx] as char;
                idx += 1;
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    break;
                }
            }
        } else {
            while idx < bytes.len() && !(bytes[idx] as char).is_whitespace() {
                idx += 1;
            }
        }
        tokens.push(inner[start..idx].to_string());
    }
    Ok(tokens)
}

fn parse_string_token(token: &str, form: &str) -> Result<String, String> {
    let Some(inner) = token.strip_prefix('"').and_then(|s| s.strip_suffix('"')) else {
        return Err(format!("{form} expects string literal paths, got {token}"));
    };
    let mut result = String::new();
    let mut chars = inner.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            let next = chars
                .next()
                .ok_or_else(|| format!("unterminated escape in string literal {token}"))?;
            result.push(next);
        } else {
            result.push(ch);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir() -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("delis-module-test-{stamp}"))
    }

    #[test]
    fn expands_required_modules_before_local_body() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("base.sl"),
            "(provide Pos zero)\n(declare-sort Pos 0)\n(declare-fun zero (Pos) Bool)\n",
        )
        .unwrap();
        fs::write(
            dir.join("main.sl"),
            "(require \"base.sl\")\n(declare-fun local (Pos) Bool)\n",
        )
        .unwrap();

        let expanded = load_module_file(dir.join("main.sl")).unwrap();
        assert!(expanded.contains("(declare-sort Pos 0)"));
        assert!(expanded.contains("(declare-fun zero (Pos) Bool)"));
        assert!(expanded.contains("(declare-fun local (Pos) Bool)"));
        assert!(!expanded.contains("(provide"));
        assert!(!expanded.contains("(require"));
    }

    #[test]
    fn rejects_provide_of_undefined_name() {
        let dir = temp_dir();
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("bad.sl"),
            "(provide missing)\n(declare-sort Pos 0)\n",
        )
        .unwrap();

        let err = load_module_file(dir.join("bad.sl")).unwrap_err();
        assert!(err
            .to_string()
            .contains("provides undefined name(s): missing"));
    }
}
