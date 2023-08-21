use crate::{
    element::{
        TmplAttr, TmplAttrKind, TmplAttrValue, TmplElement, TmplNode, TmplTextNode, TmplVirtualType,
    },
    escape::{escape_html_text, gen_lit_str},
    expr::{TmplExpr, TmplExprLevel},
    tree::TmplTree,
    TmplGroup,
};
use std::fmt;

impl fmt::Debug for TmplGroup {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (name, tree) in self.trees.iter() {
            writeln!(f, "{}", name)?;
            writeln!(f, "{}", tree)?;
        }
        Ok(())
    }
}

impl fmt::Display for TmplTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let import_strings: Vec<String> = self
            .imports
            .iter()
            .map(|x| format!(r#"<import src="{}"></import>"#, escape_html_text(x)))
            .collect();
        let sub_template_strings: Vec<String> = self
            .sub_templates
            .iter()
            .map(|(k, v)| {
                let children_strings: Vec<String> =
                    v.children.iter().map(|c| format!("{}", c)).collect();
                format!(
                    r#"<template name="{}">{}</template>"#,
                    escape_html_text(k),
                    children_strings.join("")
                )
            })
            .collect();
        let children_strings: Vec<String> = self
            .root
            .children
            .iter()
            .map(|c| format!("{}", c))
            .collect();
        write!(
            f,
            "{}{}{}",
            import_strings.join(""),
            sub_template_strings.join(""),
            children_strings.join("")
        )
    }
}

impl fmt::Display for TmplElement {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let virtual_string: String = match &self.virtual_type {
            TmplVirtualType::None => "".into(),
            TmplVirtualType::Pure => "".into(),
            TmplVirtualType::IfGroup => {
                let children_strings: Vec<String> =
                    self.children.iter().map(|c| format!("{}", c)).collect();
                return write!(f, "{}", children_strings.join(""));
            }
            TmplVirtualType::If { cond } => format!(r#" wx:if={}"#, cond),
            TmplVirtualType::Elif { cond } => format!(r#" wx:elif={}"#, cond),
            TmplVirtualType::Else => " wx:else".into(),
            TmplVirtualType::For {
                list,
                item_name,
                index_name,
                key,
            } => {
                let list = format!(r#" wx:for={}"#, list);
                let item = format!(r#" wx:for-item="{}""#, escape_html_text(item_name));
                let index = format!(r#" wx:for-index="{}""#, escape_html_text(index_name));
                let key = if let Some(key) = key {
                    format!(r#" wx:key="{}""#, escape_html_text(key))
                } else {
                    String::new()
                };
                list + &item + &index + &key
            }
            TmplVirtualType::TemplateRef { target, data } => {
                format!(r#" is={} data={}"#, target, data)
            }
            TmplVirtualType::Include { path } => format!(r#" src="{}""#, escape_html_text(path)),
            TmplVirtualType::Slot { name, props } => {
                let name = format!(r#" name={}"#, name);
                let props = if let Some(props) = props {
                    let prop_strings: Vec<String> =
                        props.iter().map(|prop| format!(" {}", prop)).collect();
                    prop_strings.join("")
                } else {
                    String::new()
                };
                name + &props
            }
        };
        let attr_strings: Vec<String> =
            self.attrs.iter().map(|attr| format!(" {}", attr)).collect();
        let children_strings: Vec<String> =
            self.children.iter().map(|c| format!("{}", c)).collect();
        let generics_seg = {
            self.generics
                .as_ref()
                .map(|list| {
                    list.iter()
                        .map(|(k, v)| format!(" generic:{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default()
        };
        let slot_props_seg = if self.slot_values.len() > 0 {
            let props: Vec<String> = self
                .slot_values
                .iter()
                .map(|(capture_name, provide_name)| {
                    format!(r#" slot:{}="{}""#, capture_name, provide_name)
                })
                .collect();
            props.join("")
        } else {
            String::new()
        };
        let extra_attr_seg = {
            self.extra_attr
                .as_ref()
                .map(|list| {
                    list.iter()
                        .map(|(k, v)| format!(" extra-attr:{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("")
                })
                .unwrap_or_default()
        };
        write!(
            f,
            "<{}{}{}{}{}{}>{}</{}>",
            &self.tag_name,
            virtual_string,
            attr_strings.join(""),
            generics_seg,
            slot_props_seg,
            extra_attr_seg,
            children_strings.join(""),
            &self.tag_name
        )
    }
}

impl fmt::Display for TmplNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TmplNode::TextNode(n) => write!(f, "{}", n),
            TmplNode::Element(n) => write!(f, "{}", n),
        }
    }
}

impl TmplExpr {
    pub(crate) fn to_expr_string(&self, allow_level: TmplExprLevel, is_js_target: bool) -> String {
        if self.level() > allow_level {
            return format!(
                "({})",
                self.to_expr_string(TmplExprLevel::Comma, is_js_target)
            );
        }
        match self {
            TmplExpr::ScopeIndex(index) => {
                if is_js_target {
                    format!("S({})", index)
                } else {
                    format!("${}", index)
                }
            }
            TmplExpr::Ident(x) => {
                if is_js_target {
                    format!("D.{}", x)
                } else {
                    format!("{}", x)
                }
            }
            TmplExpr::ToStringWithoutUndefined(x) => {
                format!("Y({})", x.to_expr_string(TmplExprLevel::Cond, is_js_target))
            }

            TmplExpr::LitUndefined => "undefined".to_string(),
            TmplExpr::LitNull => "null".to_string(),
            TmplExpr::LitStr(x) => gen_lit_str(x),
            TmplExpr::LitInt(x) => {
                format!("{}", x)
            }
            TmplExpr::LitFloat(x) => {
                format!("{}", x)
            }
            TmplExpr::LitBool(x) => {
                format!("{}", x)
            }
            TmplExpr::LitObj(x) => {
                let mut r = String::from("{}");
                let mut s: Vec<String> = vec![];
                for x in x.iter() {
                    let v_string = x.1.to_expr_string(TmplExprLevel::Cond, is_js_target);
                    match &x.0 {
                        Some(k) => s.push(format!("{}:{}", &k, v_string)),
                        None => {
                            if is_js_target {
                                if s.len() > 0 {
                                    r = format!(
                                        "Object.assign({},{{{}}},{})",
                                        r,
                                        s.join(","),
                                        v_string
                                    );
                                    s.truncate(0);
                                } else {
                                    r = format!("Object.assign({},{})", r, v_string);
                                }
                            } else {
                                s.push(format!("...{}", v_string))
                            }
                        }
                    }
                }
                let merged_s = format!("{{{}}}", s.join(","));
                if r.len() > 2 {
                    if s.len() > 0 {
                        format!("Object.assign({},{})", r, merged_s)
                    } else {
                        r
                    }
                } else {
                    merged_s
                }
            }
            TmplExpr::LitArr(x) => {
                let s: Vec<String> = x
                    .iter()
                    .map(|x| x.to_expr_string(TmplExprLevel::Cond, is_js_target))
                    .collect();
                format!("[{}]", s.join(","))
            }

            TmplExpr::StaticMember(x, y) => {
                format!(
                    "X({}).{}",
                    x.to_expr_string(TmplExprLevel::Cond, is_js_target),
                    y
                )
            }
            TmplExpr::DynamicMember(x, y) => {
                format!(
                    "X({})[{}]",
                    x.to_expr_string(TmplExprLevel::Cond, is_js_target),
                    y.to_expr_string(TmplExprLevel::Cond, is_js_target)
                )
            }
            TmplExpr::FuncCall(x, y) => {
                let s: Vec<String> = y
                    .iter()
                    .map(|x| x.to_expr_string(TmplExprLevel::Cond, is_js_target))
                    .collect();
                format!(
                    "{}({})",
                    x.to_expr_string(TmplExprLevel::Member, is_js_target),
                    s.join(",")
                )
            }

            TmplExpr::Reverse(x) => {
                format!("!{}", x.to_expr_string(TmplExprLevel::Unary, is_js_target))
            }
            TmplExpr::BitReverse(x) => {
                format!("~{}", x.to_expr_string(TmplExprLevel::Unary, is_js_target))
            }
            TmplExpr::Positive(x) => {
                format!("+{}", x.to_expr_string(TmplExprLevel::Unary, is_js_target))
            }
            TmplExpr::Negative(x) => {
                format!("-{}", x.to_expr_string(TmplExprLevel::Unary, is_js_target))
            }

            TmplExpr::Multiply(x, y) => {
                format!(
                    "{}*{}",
                    x.to_expr_string(TmplExprLevel::Multiply, is_js_target),
                    y.to_expr_string(TmplExprLevel::Unary, is_js_target)
                )
            }
            TmplExpr::Divide(x, y) => {
                format!(
                    "{}/{}",
                    x.to_expr_string(TmplExprLevel::Multiply, is_js_target),
                    y.to_expr_string(TmplExprLevel::Unary, is_js_target)
                )
            }
            TmplExpr::Mod(x, y) => {
                format!(
                    "{}%{}",
                    x.to_expr_string(TmplExprLevel::Multiply, is_js_target),
                    y.to_expr_string(TmplExprLevel::Unary, is_js_target)
                )
            }
            TmplExpr::Plus(x, y) => {
                format!(
                    "{}+{}",
                    x.to_expr_string(TmplExprLevel::Plus, is_js_target),
                    y.to_expr_string(TmplExprLevel::Multiply, is_js_target)
                )
            }
            TmplExpr::Minus(x, y) => {
                format!(
                    "{}-{}",
                    x.to_expr_string(TmplExprLevel::Plus, is_js_target),
                    y.to_expr_string(TmplExprLevel::Multiply, is_js_target)
                )
            }

            TmplExpr::Lt(x, y) => {
                format!(
                    "{}<{}",
                    x.to_expr_string(TmplExprLevel::Comparison, is_js_target),
                    y.to_expr_string(TmplExprLevel::Plus, is_js_target)
                )
            }
            TmplExpr::Gt(x, y) => {
                format!(
                    "{}>{}",
                    x.to_expr_string(TmplExprLevel::Comparison, is_js_target),
                    y.to_expr_string(TmplExprLevel::Plus, is_js_target)
                )
            }
            TmplExpr::Lte(x, y) => {
                format!(
                    "{}<={}",
                    x.to_expr_string(TmplExprLevel::Comparison, is_js_target),
                    y.to_expr_string(TmplExprLevel::Plus, is_js_target)
                )
            }
            TmplExpr::Gte(x, y) => {
                format!(
                    "{}>={}",
                    x.to_expr_string(TmplExprLevel::Comparison, is_js_target),
                    y.to_expr_string(TmplExprLevel::Plus, is_js_target)
                )
            }
            TmplExpr::Eq(x, y) => {
                format!(
                    "{}=={}",
                    x.to_expr_string(TmplExprLevel::Eq, is_js_target),
                    y.to_expr_string(TmplExprLevel::Comparison, is_js_target)
                )
            }
            TmplExpr::Ne(x, y) => {
                format!(
                    "{}!={}",
                    x.to_expr_string(TmplExprLevel::Eq, is_js_target),
                    y.to_expr_string(TmplExprLevel::Comparison, is_js_target)
                )
            }
            TmplExpr::EqFull(x, y) => {
                format!(
                    "{}==={}",
                    x.to_expr_string(TmplExprLevel::Eq, is_js_target),
                    y.to_expr_string(TmplExprLevel::Comparison, is_js_target)
                )
            }
            TmplExpr::NeFull(x, y) => {
                format!(
                    "{}!=={}",
                    x.to_expr_string(TmplExprLevel::Eq, is_js_target),
                    y.to_expr_string(TmplExprLevel::Comparison, is_js_target)
                )
            }

            TmplExpr::BitAnd(x, y) => {
                format!(
                    "{}&{}",
                    x.to_expr_string(TmplExprLevel::BitAnd, is_js_target),
                    y.to_expr_string(TmplExprLevel::Eq, is_js_target)
                )
            }
            TmplExpr::BitXor(x, y) => {
                format!(
                    "{}^{}",
                    x.to_expr_string(TmplExprLevel::BitXor, is_js_target),
                    y.to_expr_string(TmplExprLevel::BitAnd, is_js_target)
                )
            }
            TmplExpr::BitOr(x, y) => {
                format!(
                    "{}|{}",
                    x.to_expr_string(TmplExprLevel::BitOr, is_js_target),
                    y.to_expr_string(TmplExprLevel::BitXor, is_js_target)
                )
            }
            TmplExpr::LogicAnd(x, y) => {
                format!(
                    "{}&&{}",
                    x.to_expr_string(TmplExprLevel::LogicAnd, is_js_target),
                    y.to_expr_string(TmplExprLevel::BitOr, is_js_target)
                )
            }
            TmplExpr::LogicOr(x, y) => {
                format!(
                    "{}||{}",
                    x.to_expr_string(TmplExprLevel::LogicOr, is_js_target),
                    y.to_expr_string(TmplExprLevel::LogicAnd, is_js_target)
                )
            }

            TmplExpr::Cond(x, y, z) => {
                format!(
                    "{}?{}:{}",
                    x.to_expr_string(TmplExprLevel::LogicOr, is_js_target),
                    y.to_expr_string(TmplExprLevel::Cond, is_js_target),
                    z.to_expr_string(TmplExprLevel::Cond, is_js_target)
                )
            }
        }
    }
}

impl fmt::Display for TmplAttr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match &self.kind {
            TmplAttrKind::WxDirective { name } => format!("wx:{}", name),
            TmplAttrKind::Generic { name } => format!("generic:{}", name),
            TmplAttrKind::Slot => format!("slot"),
            TmplAttrKind::SlotProperty { name } => format!("{}", name),
            TmplAttrKind::Id => format!("id"),
            TmplAttrKind::Class => format!("class"),
            TmplAttrKind::Style => format!("style"),
            TmplAttrKind::PropertyOrExternalClass { name } => format!("{}", name),
            TmplAttrKind::ModelProperty { name } => format!("model:{}", name),
            TmplAttrKind::ChangeProperty { name } => format!("change:{}", name),
            TmplAttrKind::WorkletProperty { name } => format!("worklet:{}", name),
            TmplAttrKind::Data { name } => format!("data:{}", name),
            TmplAttrKind::Event {
                capture,
                catch,
                mut_bind,
                name,
            } => {
                let capture_prefix = if *capture { "capture-" } else { "" };
                let main_prefix = if *catch {
                    "catch"
                } else if *mut_bind {
                    "mut-bind"
                } else {
                    "bind"
                };
                format!("{}{}-{}", capture_prefix, main_prefix, name)
            }
            TmplAttrKind::Mark { name } => format!("mark:{}", name),
        };
        write!(f, "{}={}", &s, &self.value)
    }
}

impl fmt::Display for TmplExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_expr_string(TmplExprLevel::Comma, false))
    }
}

impl fmt::Display for TmplAttrValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TmplAttrValue::Static(v) => {
                write!(f, "\"{}\"", escape_html_text(v))
            }
            TmplAttrValue::Dynamic { expr, .. } => {
                write!(f, "\"{{{{{}}}}}\"", expr)
            }
        }
    }
}

impl fmt::Display for TmplTextNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TmplTextNode::Static(v) => {
                write!(f, "{}", escape_html_text(v))
            }
            TmplTextNode::Dynamic { expr, .. } => {
                write!(f, "{{{{{}}}}}", expr)
            }
        }
    }
}
