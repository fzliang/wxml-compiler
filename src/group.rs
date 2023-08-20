use std::{collections::HashMap, fmt::Debug};

use crate::{parse_tmpl, tree::TmplTree, TmplParseError};

pub struct TmplGroup {
    trees: HashMap<String, TmplTree>,
    scripts: HashMap<String, String>,
    has_scripts: bool,
    extra_runtime_string: String,
}

impl Debug for TmplGroup {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (name, tree) in self.trees.iter() {
            writeln!(f, "{}", name)?;
            writeln!(f, "{:?}", tree)?;
        }
        Ok(())
    }
}

impl TmplGroup {
    pub fn new() -> Self {
        Self {
            trees: HashMap::new(),
            scripts: HashMap::new(),
            has_scripts: false,
            extra_runtime_string: String::new(),
        }
    }

    pub fn add_tmpl(&mut self, path: &str, tmpl_str: &str) -> Result<(), TmplParseError> {
        let mut tmpl = parse_tmpl(tmpl_str)?;
        tmpl.path = path.to_string();
        if tmpl.get_inline_script_module_name().len() > 0 {
            self.has_scripts = true;
        }
        self.trees.insert(tmpl.path.clone(), tmpl);

        Ok(())
    }

    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn contains_template(&self, path: &str) -> bool {
        self.trees.contains_key(path)
    }
}
