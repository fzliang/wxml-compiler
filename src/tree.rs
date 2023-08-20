use std::{collections::HashMap, vec};

use crate::{
    binding_map::BindingMapCollector,
    element::{TmplElement, TmplScript, TmplVirtualType},
    path,
};

#[derive(Debug)]
pub struct TmplTree {
    pub(crate) path: String,
    pub(crate) root: TmplElement,
    pub(crate) imports: Vec<String>,
    pub(crate) includes: Vec<String>,
    pub(crate) sub_templates: HashMap<String, TmplElement>,
    pub(crate) scripts: Vec<TmplScript>,
    pub(crate) binding_map_collector: BindingMapCollector,
}

impl TmplTree {
    pub(crate) fn new() -> Self {
        Self {
            path: String::new(),
            root: TmplElement::new("", TmplVirtualType::None),
            imports: vec![],
            includes: vec![],
            sub_templates: HashMap::new(),
            scripts: vec![],
            binding_map_collector: BindingMapCollector::new(),
        }
    }

    pub(crate) fn root(&self) -> &TmplElement {
        &self.root
    }

    pub(crate) fn root_mut(&mut self) -> &mut TmplElement {
        &mut self.root
    }

    pub(crate) fn get_direct_dependencies(&self) -> Vec<String> {
        let mut ret = vec![];
        for target_path in self.imports.iter() {
            ret.push(path::resolve(&self.path, &target_path));
        }
        for target_path in self.includes.iter() {
            ret.push(path::resolve(&self.path, &target_path));
        }
        ret
    }

    pub(crate) fn get_script_dependencies(&self) -> Vec<String> {
        let mut ret = vec![];
        for script in self.scripts.iter() {
            match script {
                TmplScript::GlobalRef { rel_path, .. } => {
                    let abs_path = path::resolve(&self.path, &rel_path);
                    ret.push(abs_path);
                }
                _ => {}
            }
        }
        ret
    }

    pub(crate) fn get_inline_script_module_name(&self) -> Vec<String> {
        let mut ret = vec![];
        for script in self.scripts.iter() {
            match script {
                TmplScript::Inline { module_name, .. } => {
                    ret.push(module_name.to_string());
                }
                _ => {}
            }
        }
        ret
    }

    pub(crate) fn get_inline_script(&self, module_name: &str) -> Option<&str> {
        for script in self.scripts.iter() {
            match script {
                TmplScript::Inline {
                    module_name: m,
                    content,
                } => {
                    if module_name == m {
                        return Some(&content);
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub(crate) fn set_inline_script(&mut self, module_name: &str, new_content: &str) {
        let find_inline_script = |script: &&mut TmplScript| match script {
            TmplScript::Inline { module_name: m, .. } => module_name == m,
            _ => false,
        };
        let inline_script = self.scripts.iter_mut().find(find_inline_script);

        match inline_script {
            Some(script) => {
                if let TmplScript::Inline {
                    ref mut content, ..
                } = script
                {
                    *content = String::from(new_content)
                }
            }
            None => self.scripts.push(TmplScript::Inline {
                module_name: String::from(module_name),
                content: String::from(new_content),
            }),
        }
    }
}
