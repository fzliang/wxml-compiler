use std::{
    error::Error,
    fmt::{Debug, Display},
};

use pest::Parser;
use pest_derive::Parser;

use crate::{
    convert_tree::{convert_directives, prepare_expr_in_tree},
    element::TmplAttrValue,
    expr::TmplExpr,
    parse_segment::parse_segment,
    tree::TmplTree,
};

#[derive(Parser)]
#[grammar = "tmpl.pest"]
struct TmplParser;

// 对应 tmpl.pest 中的 text_entity
pub(crate) enum TextEntity<U> {
    Static(U),
    Dynamic(Box<TmplExpr>),
}

// read all attrs
pub(crate) enum IfType {
    None,
    If(TmplAttrValue),
    Elif(TmplAttrValue),
    Else,
}

pub struct TmplParseError {
    pub message: String,
    pub start_pos: (usize, usize),
    pub end_pos: (usize, usize),
}

impl Debug for TmplParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Template parsing error (from line {} column {} to line {} column {}) : {}",
            self.start_pos.0, self.start_pos.1, self.end_pos.0, self.end_pos.1, self.message
        )
    }
}

impl Display for TmplParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for TmplParseError {}

pub fn parse_tmpl(tmpl_str: &str) -> Result<TmplTree, TmplParseError> {
    let mut pairs = TmplParser::parse(Rule::main, tmpl_str).map_err(|e| {
        let (start_pos, end_pos) = match e.line_col {
            pest::error::LineColLocation::Pos(p) => (p, p),
            pest::error::LineColLocation::Span(start, end) => (start, end),
        };

        let message = match e.variant {
            pest::error::ErrorVariant::ParsingError {
                positives: _,
                negatives: _,
            } => String::from("Unexpected character"),
            pest::error::ErrorVariant::CustomError { message: msg } => msg,
        };

        TmplParseError {
            message,
            start_pos,
            end_pos,
        }
    })?;

    let mut tree = TmplTree::new();
    // 获取Rule::main下的Rule::segment
    let main_pair = pairs.next().unwrap();
    // 获取Rule::segment下的子节点
    let mut segment = main_pair.into_inner().next().unwrap().into_inner();
    parse_segment(tree.root_mut(), &mut segment);
    convert_directives(&mut tree);
    prepare_expr_in_tree(&mut tree);
    if let Some(pair) = segment.peek() {
        let span = pair.as_span();
        return Err(TmplParseError {
            message: String::from("Unexpected segment"),
            start_pos: span.start_pos().line_col(),
            end_pos: span.end_pos().line_col(),
        });
    }
    Ok(tree)
}
