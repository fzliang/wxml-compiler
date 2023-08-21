// The js interface

#![cfg(feature = "js_bindings")]

use wasm_bindgen::prelude::*;

use super::*;


#[wasm_bindgen(js_name="parseTmpl")]
pub fn parse_tmpl(tmpl_str: &str) -> Result<String, JsError> {
    let tmpl = crate::parse_tmpl(tmpl_str)?;

    // todo 暂时将display输出
    let ret_str = format!("{:#?}", tmpl);

    Ok(ret_str)
}
