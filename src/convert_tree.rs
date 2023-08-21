use std::collections::HashMap;

use crate::{
    binding_map::BindingMapCollector,
    element::{
        TmplAttr, TmplAttrKind, TmplAttrValue, TmplElement, TmplNode, TmplScript, TmplTextNode,
        TmplVirtualType,
    },
    expr::TmplExpr,
    tree::TmplTree,
    IfType,
};

pub(crate) fn convert_directives(tree: &mut TmplTree) {
    let TmplTree {
        path: _,
        root,
        imports,
        includes,
        sub_templates,
        binding_map_collector: _,
        scripts,
    } = tree;

    convert_nodes_directives(root, imports, includes, sub_templates, scripts)
}

fn convert_nodes_directives(
    parent: &mut TmplElement,
    imports: &mut Vec<String>,
    includes: &mut Vec<String>,
    sub_templates: &mut HashMap<String, TmplElement>,
    scripts: &mut Vec<TmplScript>,
) {
    let old_children = std::mem::replace(&mut parent.children, vec![]);
    for node in old_children.into_iter() {
        match node {
            TmplNode::TextNode(text_node) => {
                parent.children.push(TmplNode::TextNode(text_node));
            }
            TmplNode::Element(mut elem) => {
                let mut inner_depth = 0;

                let mut attr_if = IfType::None;
                let mut attr_for: Option<TmplAttrValue> = None;
                let mut attr_for_item: Option<String> = None;
                let mut attr_for_index: Option<String> = None;
                let mut attr_key: Option<String> = None;
                let mut attr_slot: Option<TmplAttrValue> = None;
                let mut slot_values: Vec<(String, String)> = Vec::with_capacity(0);
                let mut generics: Option<HashMap<String, String>> = None;

                let old_attrs = std::mem::replace(&mut elem.attrs, vec![]);
                for attr in old_attrs.into_iter() {
                    match &attr.kind {
                        TmplAttrKind::WxDirective { name } => {
                            match name.as_str() {
                                "if" => attr_if = IfType::If(attr.value),
                                "elif" => attr_if = IfType::Elif(attr.value),
                                "else" => attr_if = IfType::Else,
                                "for" | "for-items" => attr_for = Some(attr.value),
                                "for-item" => attr_for_item = Some(attr.value.static_value()),
                                "for-index" => attr_for_index = Some(attr.value.static_value()),
                                "key" => attr_key = Some(attr.value.static_value()),
                                _ => {}
                            }
                            continue;
                        }
                        TmplAttrKind::Generic { name } => {
                            if let Some(map) = &mut generics {
                                map.insert(name.to_string(), attr.value.static_value());
                            } else {
                                let mut map = HashMap::new();
                                map.insert(name.to_string(), attr.value.static_value());
                                generics = Some(map);
                            }
                            continue;
                        }
                        TmplAttrKind::SlotProperty { name } => {
                            match &attr.value {
                                TmplAttrValue::Static(s) => {
                                    if s.as_str() == "" {
                                        slot_values.push((name.to_string(), name.to_string()));
                                    } else {
                                        slot_values
                                            .push((name.to_string(), attr.value.static_value()));
                                    }
                                }
                                TmplAttrValue::Dynamic { expr, .. } => {
                                    match &**expr {
                                        TmplExpr::LitBool(true) => {
                                            slot_values.push((name.to_string(), name.to_string()));
                                        }
                                        _ => {
                                            // TODO warn dynamic value
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        TmplAttrKind::Slot => {
                            attr_slot = Some(attr.value);
                            continue;
                        }
                        _ => {}
                    }
                    elem.attrs.push(attr);
                }

                // handling special tags
                match elem.tag_name.as_str() {
                    "include" | "import" => {
                        let old_attrs = std::mem::replace(&mut elem.attrs, vec![]);
                        let mut path: Option<String> = None;
                        for attr in old_attrs.into_iter() {
                            if attr.is_property("src") {
                                if path.is_some() {
                                    // FIXME warn duplicated attr
                                } else {
                                    let p = attr.value.static_value();
                                    path = Some(p.strip_suffix(".wxml").unwrap_or(&p).to_string());
                                }
                            } else {
                                // FIXME warn unused attr
                            }
                        }
                        match path {
                            Some(path) => {
                                if elem.tag_name.as_str() == "import" {
                                    imports.push(path);
                                } else {
                                    includes.push(path.clone());
                                    elem.virtual_type = TmplVirtualType::Include { path };
                                    elem.children.clear();
                                    parent.children.push(TmplNode::Element(elem));
                                }
                            }
                            None => {
                                // FIXME warn no src attr found
                            }
                        }
                        continue;
                    }
                    "template" => {
                        let old_attrs = std::mem::replace(&mut elem.attrs, vec![]);
                        let mut name: Option<String> = None;
                        let mut target: Option<TmplAttrValue> = None;
                        let mut data: Option<TmplAttrValue> = None;
                        for attr in old_attrs.into_iter() {
                            if attr.is_property("name") {
                                if name.is_some() {
                                    // FIXME warn duplicated attr
                                } else {
                                    name = Some(attr.value.static_value());
                                }
                            } else if attr.is_property("is") {
                                if target.is_some() {
                                    // FIXME warn duplicated attr
                                } else {
                                    target = Some(attr.value)
                                }
                            } else if attr.is_property("data") {
                                if data.is_some() {
                                    // FIXME warn duplicated attr
                                } else {
                                    data = Some(attr.value);
                                }
                            } else {
                                // FIXME warn unused attr
                            }
                        }

                        match name {
                            Some(name) => {
                                if target.is_some() || data.is_some() {
                                    // FIXME warn unused attr
                                }
                                convert_nodes_directives(
                                    &mut elem,
                                    imports,
                                    includes,
                                    sub_templates,
                                    scripts,
                                );
                                sub_templates.insert(name, elem);
                                continue;
                            }
                            None => {
                                match target {
                                    Some(target) => {
                                        elem.virtual_type = TmplVirtualType::TemplateRef {
                                            target,
                                            data: match data {
                                                Some(field) => {
                                                    if let TmplAttrValue::Dynamic {
                                                        expr,
                                                        binding_map_keys,
                                                    } = field
                                                    {
                                                        let expr = match *expr {
                                                            TmplExpr::Ident(s) => {
                                                                TmplExpr::LitObj(vec![(
                                                                    Some(s.clone()),
                                                                    TmplExpr::Ident(s),
                                                                )])
                                                            }
                                                            TmplExpr::LitObj(x) => {
                                                                TmplExpr::LitObj(x)
                                                            }
                                                            _ => {
                                                                // FIXME warn must be object
                                                                TmplExpr::LitObj(vec![])
                                                            }
                                                        };
                                                        TmplAttrValue::Dynamic {
                                                            expr: Box::new(expr),
                                                            binding_map_keys,
                                                        }
                                                    } else {
                                                        // FIXME warn must be object data binding
                                                        TmplAttrValue::Dynamic {
                                                            expr: Box::new(TmplExpr::LitObj(
                                                                vec![],
                                                            )),
                                                            binding_map_keys: None,
                                                        }
                                                    }
                                                }
                                                None => TmplAttrValue::Dynamic {
                                                    expr: Box::new(TmplExpr::LitObj(vec![])),
                                                    binding_map_keys: None,
                                                },
                                            },
                                        }
                                    }
                                    None => {} // FIXME warn no src attr found
                                }
                            }
                        }
                    }
                    "slot" => {
                        let old_attrs = std::mem::replace(&mut elem.attrs, vec![]);
                        let mut name = TmplAttrValue::Static(String::new());
                        let mut props: Option<Vec<TmplAttr>> = None;
                        for attr in old_attrs.into_iter() {
                            if attr.is_property("name") {
                                name = attr.value;
                            } else if attr.is_any_property() {
                                if let Some(arr) = &mut props {
                                    arr.push(attr);
                                } else {
                                    let mut arr = vec![];
                                    arr.push(attr);
                                    props = Some(arr);
                                }
                            }
                        }
                        elem.virtual_type = TmplVirtualType::Slot { name, props };
                    }
                    "wxs" => {
                        let old_attrs = std::mem::replace(&mut elem.attrs, vec![]);
                        let mut module_name = String::new();
                        let mut src = String::new();
                        for attr in old_attrs.into_iter() {
                            if attr.is_property("module") {
                                match attr.value {
                                    TmplAttrValue::Dynamic { .. } => {
                                        // FIXME warn must be static
                                    }
                                    TmplAttrValue::Static(s) => {
                                        module_name = s;
                                    }
                                }
                            } else if attr.is_property("src") {
                                match attr.value {
                                    TmplAttrValue::Dynamic { .. } => {
                                        // FIXME warn must be static
                                    }
                                    TmplAttrValue::Static(s) => {
                                        src = s.strip_suffix(".wxs").unwrap_or(&s).to_string();
                                    }
                                }
                            } else {
                                // FIXME warn unused attr
                            }
                        }
                        if src.len() == 0 {
                            let content = match elem.children.get(0) {
                                Some(TmplNode::TextNode(TmplTextNode::Static(x))) => x.as_str(),
                                None => "",
                                _ => unreachable!(),
                            };
                            scripts.push(TmplScript::Inline {
                                module_name,
                                content: content.to_string(),
                            });
                        } else {
                            // FIXME warn unused script content
                            scripts.push(TmplScript::GlobalRef {
                                module_name,
                                rel_path: src,
                            });
                        }
                        continue;
                    }

                    _ => {}
                }
                elem.generics = generics;
                elem.slot = attr_slot;

                // a helper for generating middle virtual node
                let mut wrap_virtual_elem = |mut elem: TmplElement, virtual_type| {
                    if let TmplVirtualType::Pure = elem.virtual_type {
                        elem.virtual_type = virtual_type;
                    } else {
                        let mut p = TmplElement::new("block", virtual_type);
                        p.append_element(elem);
                        elem = p;
                        inner_depth += 1;
                    }
                    elem
                };

                // handling if
                match attr_if {
                    IfType::None => {}
                    IfType::If(attr_if) => {
                        let virtual_type = TmplVirtualType::If { cond: attr_if };
                        elem = wrap_virtual_elem(elem, virtual_type);
                        elem = wrap_virtual_elem(elem, TmplVirtualType::IfGroup);
                    }
                    IfType::Elif(attr_if) => {
                        let virtual_type = TmplVirtualType::Elif { cond: attr_if };
                        if let Some(last) = parent.children.last_mut() {
                            if let TmplNode::Element(last) = last {
                                if let TmplVirtualType::IfGroup = last.virtual_type {
                                    convert_nodes_directives(
                                        &mut elem,
                                        imports,
                                        includes,
                                        sub_templates,
                                        scripts,
                                    );
                                    elem = wrap_virtual_elem(elem, virtual_type);
                                    last.append_element(elem);
                                    // FIXME here should display a warning if <for> is found
                                    continue;
                                }
                            }
                        }
                        // FIXME here should display a warning if no matching <if> found
                        elem = wrap_virtual_elem(elem, virtual_type);
                        elem = wrap_virtual_elem(elem, TmplVirtualType::IfGroup);
                    }
                    IfType::Else => {
                        let virtual_type = TmplVirtualType::Else;
                        if let Some(last) = parent.children.last_mut() {
                            if let TmplNode::Element(last) = last {
                                if let TmplVirtualType::IfGroup = last.virtual_type {
                                    convert_nodes_directives(
                                        &mut elem,
                                        imports,
                                        includes,
                                        sub_templates,
                                        scripts,
                                    );
                                    elem = wrap_virtual_elem(elem, virtual_type);
                                    last.append_element(elem);
                                    // FIXME here should display a warning if <for> is found
                                    continue;
                                }
                            }
                        }
                        // FIXME here should display a warning if no matching <if> found
                        elem = wrap_virtual_elem(elem, virtual_type);
                        elem = wrap_virtual_elem(elem, TmplVirtualType::IfGroup);
                    }
                }

                // handling for
                if let Some(attr_for) = attr_for {
                    let item_name = attr_for_item.unwrap_or("item".into());
                    let index_name = attr_for_index.unwrap_or("index".into());
                    let virtual_type = TmplVirtualType::For {
                        list: attr_for,
                        item_name,
                        index_name,
                        key: attr_key,
                    };
                    elem = wrap_virtual_elem(elem, virtual_type);
                }

                // recurse into children
                let mut next = &mut elem;
                for _ in 0..inner_depth {
                    next = match next.children.first_mut().unwrap() {
                        TmplNode::Element(elem) => elem,
                        TmplNode::TextNode(_) => unreachable!(),
                    }
                }
                convert_nodes_directives(next, imports, includes, sub_templates, scripts);

                // eliminate pure virtual node
                let is_pure_virtual = if let TmplVirtualType::Pure = elem.virtual_type {
                    true
                } else {
                    false
                };
                if is_pure_virtual && elem.slot.is_none() {
                    for child in elem.children.iter_mut() {
                        match child {
                            TmplNode::TextNode(..) => {}
                            TmplNode::Element(x) => {
                                x.slot_values = slot_values.clone();
                            }
                        }
                    }
                    parent.children.append(&mut elem.children);
                } else {
                    elem.slot_values = slot_values;
                    parent.children.push(TmplNode::Element(elem));
                }
            }
        }
    }
}

pub(crate) fn prepare_expr_in_tree(tree: &mut TmplTree) {
    let scope_names = tree
        .scripts
        .iter()
        .map(|script| match script {
            TmplScript::Inline { module_name, .. } => module_name.to_string(),
            TmplScript::GlobalRef { module_name, .. } => module_name.to_string(),
        })
        .collect();
    prepare_node_expr_in_tree(
        &mut tree.root,
        &mut tree.binding_map_collector,
        &scope_names,
        false,
    );
    for tmpl in tree.sub_templates.values_mut() {
        prepare_node_expr_in_tree(tmpl, &mut BindingMapCollector::new(), &scope_names, true);
    }
}

fn prepare_attr_value(
    v: &mut TmplAttrValue,
    bmc: &mut BindingMapCollector,
    scope_names: &Vec<String>,
    should_disable: bool,
) {
    match v {
        TmplAttrValue::Static(_) => {}
        TmplAttrValue::Dynamic {
            expr,
            binding_map_keys,
        } => {
            *binding_map_keys = expr.get_binding_map_keys(bmc, scope_names, should_disable);
        }
    }
}

fn prepare_node_expr_in_tree(
    parent: &mut TmplElement,
    bmc: &mut BindingMapCollector,
    scope_names: &Vec<String>,
    should_disable: bool,
) {
    for node in parent.children.iter_mut() {
        match node {
            TmplNode::TextNode(ref mut text_node) => match text_node {
                TmplTextNode::Static(_) => {}
                TmplTextNode::Dynamic {
                    expr,
                    binding_map_keys,
                } => {
                    *binding_map_keys = expr.get_binding_map_keys(bmc, scope_names, should_disable);
                }
            },
            TmplNode::Element(ref mut elem) => {
                let should_disable = match &elem.virtual_type {
                    TmplVirtualType::None => should_disable,
                    _ => true,
                };
                let mut new_scope_names = None;
                match &mut elem.virtual_type {
                    TmplVirtualType::None => {}
                    TmplVirtualType::Pure => {}
                    TmplVirtualType::IfGroup => {}
                    TmplVirtualType::If { cond } => {
                        prepare_attr_value(cond, bmc, scope_names, true);
                    }
                    TmplVirtualType::Elif { cond } => {
                        prepare_attr_value(cond, bmc, scope_names, true);
                    }
                    TmplVirtualType::Else => {}
                    TmplVirtualType::For {
                        list,
                        item_name,
                        index_name,
                        key: _,
                    } => {
                        prepare_attr_value(list, bmc, scope_names, true);
                        let s_len = scope_names.len();
                        let mut s = scope_names.clone();
                        let new_item_name = format!("${}", s_len);
                        let new_index_name = format!("${}", s_len + 1);
                        s.push(std::mem::replace(item_name, new_item_name));
                        s.push(std::mem::replace(index_name, new_index_name));
                        new_scope_names = Some(s);
                    }
                    TmplVirtualType::TemplateRef { target, data } => {
                        prepare_attr_value(target, bmc, scope_names, true);
                        prepare_attr_value(data, bmc, scope_names, true);
                    }
                    TmplVirtualType::Include { path: _ } => {}
                    TmplVirtualType::Slot { name, props } => {
                        prepare_attr_value(name, bmc, scope_names, true);
                        if let Some(props) = props {
                            for attr in props.iter_mut() {
                                prepare_attr_value(&mut attr.value, bmc, scope_names, true);
                            }
                        }
                    }
                }
                if elem.slot_values.len() > 0 {
                    let s_len = scope_names.len();
                    let mut s = scope_names.clone();
                    for (index, (_, provide_name)) in elem.slot_values.iter_mut().enumerate() {
                        let new_provide_name = format!("${}", s_len + index);
                        s.push(std::mem::replace(provide_name, new_provide_name));
                    }
                    new_scope_names = Some(s);
                }
                let scope_names_ref = new_scope_names.as_ref().unwrap_or(scope_names);
                for attr in elem.attrs.iter_mut() {
                    prepare_attr_value(&mut attr.value, bmc, scope_names_ref, should_disable);
                }
                if let Some(slot) = elem.slot.as_mut() {
                    prepare_attr_value(slot, bmc, scope_names_ref, should_disable);
                }
                prepare_node_expr_in_tree(elem, bmc, scope_names_ref, should_disable);
            }
        }
    }
}
