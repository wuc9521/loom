use rustc_hash::FxHashMap as HashMap;

use sygus_parser::{Sort, SyGuSCmd};

#[derive(Debug, Clone, Default)]
pub struct SortEnv {
    aliases: HashMap<String, Sort>,
}

impl SortEnv {
    pub fn from_cmds(cmds: &[SyGuSCmd]) -> Self {
        let aliases = cmds
            .iter()
            .filter_map(|cmd| match cmd {
                SyGuSCmd::DefineSort(name, params, sort) if params.is_empty() => {
                    Some((name.clone(), sort.clone()))
                }
                _ => None,
            })
            .collect();
        Self { aliases }
    }

    pub fn resolve<'a>(&'a self, sort: &'a Sort) -> Option<&'a Sort> {
        match sort {
            Sort::Simple(id) => self.aliases.get(id.get_name()),
            _ => None,
        }
    }

    pub fn bv_width(&self, sort: &Sort) -> Result<usize, String> {
        let text = sort.to_string();
        if let Some(width) = text
            .strip_prefix("(_ BitVec ")
            .and_then(|rest| rest.strip_suffix(')'))
            .and_then(|width| width.parse::<usize>().ok())
        {
            return Ok(width);
        }
        self.resolve(sort)
            .ok_or_else(|| format!("expected BitVec sort, got {sort}"))
            .and_then(|resolved| self.bv_width(resolved))
    }
}
