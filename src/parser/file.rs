use crate::parser::{LSParseError, LSParser, LSCmd, Rule, LSFile};
use itertools::Itertools;
use pest::Parser;



impl LSFile {
    pub fn new() -> Self {
        LSFile { cmds: Vec::new() }
    }
    pub fn push(&mut self, cmd: LSCmd) {
        self.cmds.push(cmd);
    }
    /// parse a complete LS file content
    pub fn parse(content: &str) -> Result<LSFile, LSParseError> {
        let [file]: [_; 1] = LSParser::parse(Rule::LSProg, content)?
            .collect_vec()
            .try_into()
            .unwrap();
        let mut ls_file = LSFile::new();

        // collect all commands
        let cmds_raw: Vec<_> = file
            .into_inner()
            .filter(|c| c.as_rule() != Rule::EOI)
            .map(|c| {
                let [cmd, ..]: [_; 1] = c.into_inner().collect_vec().try_into().unwrap();
                cmd
            })
            .collect();

        // process each command
        for cmd in cmds_raw {
            let parsed_cmd = LSCmd::parse(cmd)?;
            ls_file.push(parsed_cmd);
        }

        Ok(ls_file)
    }
}
