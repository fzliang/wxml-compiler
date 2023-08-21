// The js interface

#![cfg(feature = "js_bindings")]

use wasm_bindgen::prelude::*;

use super::*;

#[wasm_bindgen]
pub struct TmplGroup {
    group: crate::TmplGroup,
    names: Vec<String>,
}

#[wasm_bindgen]
impl TmplGroup {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            group: crate::TmplGroup::new(),
            names: vec![],
        }
    }

    #[wasm_bindgen(js_name = "addTmpl")]
    pub fn add_tmpl(&mut self, path: &str, tmpl_str: &str) -> Result<usize, JsError> {
        let path = path::normalize(path);
        self.group.add_tmpl(&path, tmpl_str)?;
        if let Some(x) = self.names.iter().position(|x| x.as_str() == path.as_str()) {
            Ok(x)
        } else {
            self.names.push(path);
            Ok(self.names.len() - 1)
        }
    }

    #[wasm_bindgen(js_name = "addScript")]
    pub fn add_script(&mut self, path: &str, tmpl_str: &str) -> Result<(), JsError> {
        let path = path::normalize(path);
        self.group.add_script(&path, tmpl_str)?;
        Ok(())
    }
}

#[wasm_bindgen(js_name="parseTmpl")]
pub fn parse_tmpl(tmpl_str: &str) -> Result<String, JsError> {
    let tmpl = crate::parse_tmpl(tmpl_str)?;

    // todo 暂时将display输出
    let ret_str = format!("{}", tmpl);

    Ok(ret_str)
}
