use std::collections::HashMap;

use crate::{binding_map::BindingMapKeys, expr::TmplExpr, utils::dash_to_camel};

#[derive(Debug)]
pub struct TmplElement {
    pub(crate) virtual_type: TmplVirtualType,
    pub(crate) tag_name: String,
    pub(crate) attrs: Vec<TmplAttr>,
    pub(crate) children: Vec<TmplNode>,
    pub(crate) generics: Option<HashMap<String, String>>,
    pub(crate) extra_attr: Option<HashMap<String, String>>,
    pub(crate) slot: Option<TmplAttrValue>,
    pub(crate) slot_values: Vec<(String, String)>,
}

#[derive(Debug)]
pub(crate) enum TmplVirtualType {
    None,
    Pure,
    For {
        list: TmplAttrValue,
        item_name: String,
        index_name: String,
        key: Option<String>,
    },
    IfGroup,
    If {
        cond: TmplAttrValue,
    },
    Elif {
        cond: TmplAttrValue,
    },
    Else,
    TemplateRef {
        target: TmplAttrValue,
        data: TmplAttrValue,
    },
    Include {
        path: String,
    },
    Slot {
        name: TmplAttrValue,
        props: Option<Vec<TmplAttr>>,
    },
}

#[derive(Debug)]
pub(crate) struct TmplAttr {
    pub(crate) kind: TmplAttrKind,
    pub(crate) value: TmplAttrValue,
}

#[derive(Debug)]
pub(crate) enum TmplAttrKind {
    WxDirective {
        name: String,
    },
    Generic {
        name: String,
    },
    Slot,
    SlotProperty {
        name: String,
    },
    Id,
    Class,
    Style,
    PropertyOrExternalClass {
        name: String,
    },
    ModelProperty {
        name: String,
    },
    ChangeProperty {
        name: String,
    },
    WorkletProperty {
        name: String,
    },
    Data {
        name: String,
    },
    Mark {
        name: String,
    },
    Event {
        capture: bool,
        catch: bool,
        mut_bind: bool,
        name: String,
    },
}

#[derive(Debug)]
pub(crate) enum TmplAttrValue {
    Static(String),
    Dynamic {
        expr: Box<TmplExpr>,
        binding_map_keys: Option<BindingMapKeys>,
    },
}

#[derive(Debug)]
pub(crate) enum TmplScript {
    Inline {
        module_name: String,
        content: String,
    },
    GlobalRef {
        module_name: String,
        rel_path: String,
    },
}

#[derive(Debug)]
pub(crate) enum TmplNode {
    TextNode(TmplTextNode),
    Element(TmplElement),
}

#[derive(Debug)]
pub(crate) enum TmplTextNode {
    Static(String),
    Dynamic {
        expr: Box<TmplExpr>,
        binding_map_keys: Option<BindingMapKeys>,
    },
}

impl TmplElement {
    pub(crate) fn new(tag_name: &str, virtual_type: TmplVirtualType) -> Self {
        Self {
            virtual_type: virtual_type,
            tag_name: String::from(tag_name),
            attrs: vec![],
            children: vec![],
            generics: None,
            extra_attr: None,
            slot: None,
            slot_values: Vec::with_capacity(0),
        }
    }

    pub(crate) fn tag_name_is(&self, tag_name: &str) -> bool {
        self.tag_name == tag_name
    }

    pub(crate) fn add_attr(&mut self, name: &str, value: TmplAttrValue) {
        let kind = if let Some((prefix, name)) = name.split_once(":") {
            let name = name.to_string();
            match prefix {
                "wx" => TmplAttrKind::WxDirective { name },
                "generic" => TmplAttrKind::Generic {
                    name: name.to_ascii_lowercase(),
                },
                "slot" => TmplAttrKind::SlotProperty {
                    name: dash_to_camel(&name.to_ascii_lowercase()),
                },
                "model" => TmplAttrKind::ModelProperty {
                    name: dash_to_camel(&name),
                },
                "change" => TmplAttrKind::ChangeProperty {
                    name: dash_to_camel(&name),
                },
                "worklet" => TmplAttrKind::WorkletProperty {
                    name: dash_to_camel(&name),
                },
                "data" => TmplAttrKind::Data {
                    name: dash_to_camel(&name),
                },
                "bind" => TmplAttrKind::Event {
                    capture: false,
                    catch: false,
                    mut_bind: false,
                    name,
                },
                "mut-bind" => TmplAttrKind::Event {
                    capture: false,
                    catch: false,
                    mut_bind: true,
                    name,
                },
                "catch" => TmplAttrKind::Event {
                    capture: false,
                    catch: true,
                    mut_bind: false,
                    name,
                },
                "capture-bind" => TmplAttrKind::Event {
                    capture: true,
                    catch: false,
                    mut_bind: false,
                    name,
                },
                "capture-mut-bind" => TmplAttrKind::Event {
                    capture: true,
                    catch: false,
                    mut_bind: true,
                    name,
                },
                "capture-catch" => TmplAttrKind::Event {
                    capture: true,
                    catch: true,
                    mut_bind: false,
                    name,
                },
                "mark" => TmplAttrKind::Mark { name },
                _ => {
                    return;
                }
            }
        } else {
            match name {
                "slot" => TmplAttrKind::Slot,
                "id" => TmplAttrKind::Id,
                "class" => TmplAttrKind::Class,
                "style" => TmplAttrKind::Style,

                name if name.starts_with("data-") => {
                    let camel_name = dash_to_camel(&name[5..].to_ascii_lowercase());
                    TmplAttrKind::Data { name: camel_name }
                }
                name => TmplAttrKind::PropertyOrExternalClass {
                    name: name.to_string(),
                },
            }
        };

        self.attrs.push(TmplAttr { kind, value })
    }

    pub(crate) fn append_text_node(&mut self, child: TmplTextNode) {
        self.children.push(TmplNode::TextNode(child));
    }

    pub(crate) fn append_element(&mut self, child: TmplElement) {
        self.children.push(TmplNode::Element(child));
    }
}

impl TmplAttr {
    pub(crate) fn is_property(&self, n: &str) -> bool {
        match &self.kind {
            TmplAttrKind::PropertyOrExternalClass { name } if name.as_str() == n => true,
            _ => false,
        }
    }

    pub(crate) fn is_any_property(&self) -> bool {
        match &self.kind {
            TmplAttrKind::PropertyOrExternalClass { .. } => true,
            _ => false,
        }
    }
}

impl TmplAttrValue {
    pub(crate) fn static_value(self) -> String {
        match self {
            TmplAttrValue::Static(s) => s,
            TmplAttrValue::Dynamic { .. } => String::new(),
        }
    }
}

impl TmplTextNode {
    pub(crate) fn new_static(content: String) -> Self {
        Self::Static(content)
    }

    pub(crate) fn new_dynamic(expr: Box<TmplExpr>) -> Self {
        Self::Dynamic {
            expr,
            binding_map_keys: None,
        }
    }
}
