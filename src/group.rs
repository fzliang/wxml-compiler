use std::collections::HashMap;

use crate::{parse_tmpl, tree::TmplTree, TmplParseError};

pub struct TmplGroup {
    pub(crate) trees: HashMap<String, TmplTree>,
    pub(crate) scripts: HashMap<String, String>,
    pub(crate) has_scripts: bool,
    pub(crate) extra_runtime_string: String,
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

    /// Add a script segment into the group.
    ///
    /// The `content` must be valid JavaScript file content.
    /// `require` and `exports` can be visited in this JavaScript segment, similar to Node.js.
    pub fn add_script(&mut self, path: &str, content: &str) -> Result<(), TmplParseError> {
        self.scripts.insert(path.to_string(), content.to_string());
        self.has_scripts = true;
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.trees.len()
    }

    pub fn contains_template(&self, path: &str) -> bool {
        self.trees.contains_key(path)
    }
}
