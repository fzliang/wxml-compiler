use std::borrow::Cow;

use crate::{
    element::{TmplAttrValue, TmplElement, TmplTextNode, TmplVirtualType},
    entities::decode,
    expr::TmplExpr,
    parse_text_entity::parse_text_entity,
    Rule, TextEntity,
};
use pest::iterators::{Pair, Pairs};

// 解析Rule::segment的子节点
// 包含 tag | text_node
pub(crate) fn parse_segment(target: &mut TmplElement, pairs: &mut Pairs<'_, Rule>) {
    while let Some(pair) = pairs.peek() {
        match pair.as_rule() {
            Rule::tag => parse_tag(target, pair, pairs),
            Rule::text_node => {
                parse_text_node(target, pair);
                pairs.next();
            }
            _ => unreachable!(),
        }
    }
}

fn parse_tag(target: &mut TmplElement, pair: Pair<'_, Rule>, pairs: &mut Pairs<'_, Rule>) {
    let mut tag_pairs = pair.into_inner();
    if let Some(pair) = tag_pairs.next() {
        let read_attr = |mut pairs: Pairs<Rule>| {
            let name = pairs.next().unwrap();
            let value = match pairs.next() {
                None => TmplAttrValue::Dynamic {
                    expr: Box::new(TmplExpr::LitBool(true)),
                    binding_map_keys: None,
                },
                Some(x) => {
                    let value = x.into_inner().next().unwrap();
                    match parse_text_entity(value) {
                        TextEntity::Static(s) => TmplAttrValue::Static(s),
                        TextEntity::Dynamic(expr) => TmplAttrValue::Dynamic {
                            expr,
                            binding_map_keys: None,
                        },
                    }
                }
            };
            (name.as_str().to_string(), value)
        };
        match pair.as_rule() {
            Rule::wxs_script_tag_begin => {
                let mut elem = TmplElement::new("wxs", TmplVirtualType::Pure);
                let pair = tag_pairs.next().unwrap();
                match pair.as_rule() {
                    Rule::wxs_script_tag => {
                        let mut wxs_pairs = pair.into_inner();
                        while let Some(pair) = wxs_pairs.next() {
                            match pair.as_rule() {
                                Rule::attr => {
                                    let pairs = pair.into_inner();
                                    let (name, value) = read_attr(pairs);
                                    elem.add_attr(name.as_str(), value);
                                }
                                Rule::wxs_script_body => {
                                    let text: String = pair
                                        .into_inner()
                                        .map(|pair| match pair.as_rule() {
                                            Rule::entity => decode(pair.as_str()),
                                            Rule::pure_text => Cow::Borrowed(pair.as_str()),
                                            _ => unreachable!(),
                                        })
                                        .collect();
                                    elem.append_text_node(TmplTextNode::Static(text));
                                    break;
                                }
                                _ => unreachable!(),
                            }
                        }
                    }
                    _ => unreachable!(),
                }
                pairs.next();
                target.append_element(elem);
            }
            Rule::tag_begin => {
                let mut elem = {
                    let mut pairs = pairs.into_iter();
                    let tag_name = pairs.next().unwrap().as_str();
                    let virtual_type = if tag_name == "block" {
                        TmplVirtualType::Pure
                    } else {
                        TmplVirtualType::None
                    };
                    let mut elem = TmplElement::new(tag_name, virtual_type);
                    while let Some(pair) = pairs.next() {
                        let pairs = pair.into_inner();
                        let (name, value) = read_attr(pairs);
                        elem.add_attr(name.as_str(), value);
                    }
                    elem
                };
                if let Some(pair) = tag_pairs.next() {
                    match pair.as_rule() {
                        Rule::self_close => {}
                        _ => unreachable!(),
                    }
                    pairs.next();
                } else {
                    pairs.next();
                    parse_segment(&mut elem, pairs);
                }
                target.append_element(elem);
            }
            Rule::tag_end => {
                let tag_name_matched = {
                    let mut pairs = pair.into_inner();
                    let tag_name = pairs.next().unwrap().as_str();
                    target.tag_name_is(tag_name)
                };
                if tag_name_matched {
                    pairs.next();
                }
                return;
            }
            _ => unreachable!(),
        }
    } else {
        pairs.next();
    }
}

fn parse_text_node(target: &mut TmplElement, pair: Pair<'_, Rule>) {
    match parse_text_entity(pair) {
        TextEntity::Static(s) => {
            if s.trim() != "" {
                target.append_text_node(TmplTextNode::new_static(s))
            }
        }
        TextEntity::Dynamic(expr) => target.append_text_node(TmplTextNode::new_dynamic(expr)),
    }
}
