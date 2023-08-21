use std::borrow::Cow;

use crate::{entities, expr::*, parse_common_op, parser::*};
use pest::iterators::Pair;

pub(crate) fn parse_text_entity(pair: Pair<'_, Rule>) -> TextEntity<String> {
    let mut is_dynamic = false;
    let segs: Vec<TextEntity<Cow<str>>> = pair
        .into_inner()
        .map(|pair: Pair<'_, Rule>| {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::expr_or_obj => {
                    is_dynamic = true;
                    TextEntity::Dynamic(parse_expr_or_obj(pair))
                }
                Rule::entity => TextEntity::Static(entities::decode(pair.as_str())),
                Rule::pure_text => TextEntity::Static(Cow::Borrowed(pair.as_str())),
                _ => unreachable!(),
            }
        })
        .collect();

    let has_multi_segs = segs.len() > 1;

    if is_dynamic {
        let mut segs = segs.into_iter();
        let mut cur = match segs.next().unwrap() {
            TextEntity::Static(s) => Box::new(TmplExpr::LitStr(s.to_string().into())),
            TextEntity::Dynamic(expr) => {
                if has_multi_segs {
                    Box::new(TmplExpr::ToStringWithoutUndefined(expr))
                } else {
                    expr
                }
            }
        };
        for seg in segs {
            if let TextEntity::Static(dest) = &seg {
                if let TmplExpr::Plus(_, cur) = &mut *cur {
                    if let TmplExpr::LitStr(src) = &mut **cur {
                        **cur = TmplExpr::LitStr((src.clone() + &dest).into());
                        continue;
                    }
                } else if let TmplExpr::LitStr(src) = &mut *cur {
                    *cur = TmplExpr::LitStr((src.clone() + &dest).into());
                    continue;
                }
            }
            let next = match seg {
                TextEntity::Static(s) => Box::new(TmplExpr::LitStr(s.to_string().into())),
                TextEntity::Dynamic(expr) => {
                    if has_multi_segs {
                        Box::new(TmplExpr::ToStringWithoutUndefined(expr))
                    } else {
                        expr
                    }
                }
            };
            cur = Box::new(TmplExpr::Plus(cur, next));
        }
        TextEntity::Dynamic(cur)
    } else {
        let s: Vec<&str> = segs
            .iter()
            .map(|x| {
                if let TextEntity::Static(x) = x {
                    let x: &str = &x;
                    x
                } else {
                    unreachable!()
                }
            })
            .collect();
        TextEntity::Static(s.join(""))
    }
}

// expr_or_obj = !{ &(ident ~ (":" | ",") | spread) ~ obj_body | cond }
pub(crate) fn parse_expr_or_obj(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::cond => parse_cond(pair),
        Rule::obj_body => parse_obj(pair),
        _ => unreachable!(),
    }
}

// cond = { or_expr ~ ("?" ~ cond ~ ":" ~ cond)? }
fn parse_cond(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let mut pairs = pair.into_inner();
    let mut ret = parse_or(pairs.next().unwrap());
    if let Some(true_pair) = pairs.next() {
        let false_pair = pairs.next().unwrap();
        ret = Box::new(TmplExpr::Cond(
            ret,
            parse_cond(true_pair),
            parse_cond(false_pair),
        ))
    }
    ret
}

// obj_body = { lit_obj_item ~ ("," ~ lit_obj_item )* }
// lit_obj = { "{" ~ (lit_obj_item ~ ("," ~ lit_obj_item )*)? ~ "}" }
// lit_obj_item = {
//     ident ~ (":" ~ cond)?
//     | lit_str ~ ":" ~ cond
//     | spread ~ cond
// }
fn parse_obj(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let pairs = pair.into_inner();
    let obj = pairs
        .map(|x| {
            let mut pairs = x.into_inner();
            let pair = pairs.next().unwrap();
            let k: Option<String> = match pair.as_rule() {
                Rule::ident => Some(pair.as_str().to_string()),
                Rule::lit_str => Some(parse_str_content(pair)),
                Rule::spread => None,
                _ => unreachable!(),
            };
            let v = if let Some(x) = pairs.next() {
                *parse_cond(x)
            } else {
                TmplExpr::Ident(k.clone().unwrap())
            };
            (k, v)
        })
        .collect();

    Box::new(TmplExpr::LitObj(obj))
}

// value = {
//     "(" ~ cond ~ ")"
//     | lit_str
//     | lit_number
//     | lit_obj
//     | lit_arr
//     | ident
// }
fn parse_value(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let pair = pair.into_inner().next().unwrap();
    match pair.as_rule() {
        Rule::cond => parse_cond(pair),
        Rule::lit_str => parse_str(pair),
        Rule::lit_number => parse_number(pair),
        Rule::lit_obj => parse_obj(pair),
        Rule::lit_arr => parse_arr(pair),
        Rule::ident => parse_ident_or_keyword(pair),
        _ => unreachable!(),
    }
}

// ident = ${ (ASCII_ALPHA | "_" | "$") ~ (ASCII_ALPHA | "_" | "$" | ASCII_DIGIT)* }
fn parse_ident_or_keyword(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let name = pair.as_str();
    match name {
        "undefined" => Box::new(TmplExpr::LitUndefined),
        "null" => Box::new(TmplExpr::LitNull),
        "true" => Box::new(TmplExpr::LitBool(true)),
        "false" => Box::new(TmplExpr::LitBool(false)),
        x => Box::new(TmplExpr::Ident(x.to_string())),
    }
}

fn parse_str(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    Box::new(TmplExpr::LitStr(parse_str_content(pair)))
}

// lit_str = ${
//     "\"" ~ (lit_str_escaped | lit_str_q)* ~ "\""
//     | "'" ~ (lit_str_escaped | lit_str_sq)* ~ "'"
// }
// lit_str_q = @{ (!"\"" ~ !"\\" ~ ANY)+ }
// lit_str_sq = @{ (!"\'" ~ !"\\" ~ ANY)+ }
// lit_str_escaped = @{ "\\" ~ ("u" ~ ASCII_DIGIT{4} | "x" ~ ASCII_DIGIT{2} | ANY) }
fn parse_str_content(pair: Pair<'_, Rule>) -> String {
    pair.into_inner()
        .map(|pair| match pair.as_rule() {
            Rule::lit_str_escaped => {
                let s = pair.as_str();
                let c = match &s[1..2] {
                    "r" => '\r',
                    "n" => '\n',
                    "t" => '\t',
                    "b" => '\x08',
                    "f" => '\x0C',
                    "v" => '\x0B',
                    "0" => '\0',
                    "'" => '\'',
                    "\"" => '"',
                    "x" | "u" => {
                        std::char::from_u32(s[2..].parse::<u32>().unwrap()).unwrap_or('\0')
                    }
                    _ => s.chars().nth(1).unwrap(),
                };
                c.to_string()
            }
            _ => pair.as_str().to_string(),
        })
        .collect()
}

// lit_number = ${
//     "0x" ~ lit_number_hex
//     | "0" ~ lit_number_oct
//     | lit_number_dec ~ lit_number_float? ~ lit_number_e?
// }
// lit_number_hex = @{ ASCII_HEX_DIGIT+ }
// lit_number_oct = @{ ASCII_OCT_DIGIT+ }
// lit_number_dec = @{ ASCII_DIGIT+ }
// lit_number_float = @{ "." ~ ASCII_DIGIT* }
// lit_number_e = @{ "e" ~ "-"? ~ ASCII_DIGIT* }
fn parse_number(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let mut pairs = pair.into_inner();
    let main = pairs.next().unwrap();
    let num = match main.as_rule() {
        Rule::lit_number_hex => {
            TmplExpr::LitInt(i32::from_str_radix(main.as_str(), 16).unwrap_or(0))
        }
        Rule::lit_number_oct => {
            TmplExpr::LitInt(i32::from_str_radix(main.as_str(), 8).unwrap_or(0))
        }
        Rule::lit_number_dec => {
            if let Some(next) = pairs.next() {
                let mut s = main.as_str().to_string() + next.as_str();
                if let Some(next) = pairs.next() {
                    s += next.as_str()
                }
                TmplExpr::LitFloat(s.parse::<f64>().unwrap_or(0.))
            } else {
                TmplExpr::LitInt(i32::from_str_radix(main.as_str(), 10).unwrap_or(0))
            }
        }
        _ => unreachable!(),
    };

    Box::new(num)
}

// lit_arr = { "[" ~ (cond ~ ("," ~ cond)*)? ~ "]" }
fn parse_arr(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let pairs = pair.into_inner();
    let arr = pairs.map(|x| *parse_cond(x)).collect();
    Box::new(TmplExpr::LitArr(arr))
}

// member = { value ~ (static_member | dynamic_member | func_call)* }
// static_member = { "." ~ ident }
// dynamic_member = { "[" ~ cond ~ "]" }
// func_call = { "(" ~ (cond ~ ("," ~ cond)*)? ~ ")"
fn parse_member(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let mut pairs = pair.into_inner();
    let mut ret = parse_value(pairs.next().unwrap());
    while let Some(op) = pairs.next() {
        match op.as_rule() {
            Rule::static_member => {
                let next = op.into_inner().next().unwrap();
                ret = Box::new(TmplExpr::StaticMember(ret, next.as_str().to_string()))
            }
            Rule::dynamic_member => {
                let next = parse_cond(op.into_inner().next().unwrap());
                ret = Box::new(TmplExpr::DynamicMember(ret, next))
            }
            Rule::func_call => {
                let next = op.into_inner().map(|next| *parse_cond(next)).collect();
                ret = Box::new(TmplExpr::FuncCall(ret, next))
            }
            _ => unreachable!(),
        }
    }
    ret
}
// unary = { (reverse | bit_reverse | positive | negative) ~ unary | member }
// reverse = { "!" }
// bit_reverse = { "~" }
// positive = { "+" }
// negative = { "-" }
fn parse_unary(pair: Pair<'_, Rule>) -> Box<TmplExpr> {
    let mut pairs = pair.into_inner();
    let op = pairs.next().unwrap();
    if op.as_rule() == Rule::member {
        return parse_member(op);
    }
    let next = parse_unary(pairs.next().unwrap());
    let ret = Box::new(match op.as_rule() {
        Rule::reverse => TmplExpr::Reverse(next),
        Rule::bit_reverse => TmplExpr::BitReverse(next),
        Rule::positive => TmplExpr::Positive(next),
        Rule::negative => TmplExpr::Negative(next),
        _ => unreachable!(),
    });

    ret
}

// or_expr = { and_expr ~ (or ~ and_expr)* }
// or = { "||" }
// and_expr = { bit_or_expr ~ (and ~ bit_or_expr)* }
// and = { "&&" }
// bit_or_expr = { bit_xor_expr ~ (bit_or ~ bit_xor_expr)* }
// bit_or = { !"||" ~ "|" }
// bit_xor_expr = { bit_and_expr ~ (bit_xor ~ bit_and_expr)* }
// bit_xor = { "^" }
// bit_and_expr = { equal ~ (bit_and ~ equal)* }
// bit_and = { !"&&" ~ "&" }
// equal = { cmp ~ ((eq_full | ne_full | eq | ne) ~ cmp)* }
// eq = { "==" }
// ne = { "!=" }
// eq_full = { "===" }
// ne_full = { "!==" }
// cmp = { plus_minus ~ ((lte | gte | lt | gt ) ~ plus_minus)* }
// lt = { "<" }
// gt = { ">" }
// lte = { "<=" }
// gte = { ">=" }
// plus_minus = { multi_div ~ ((plus | minus) ~ multi_div)* }
// plus = { "+" }
// minus = { "-" }
// multi_div = { unary ~ ((multi | div | rem) ~ unary)* }
// multi = { "*" }
// div = { "/" }
// rem = { "%" }
parse_common_op!(parse_multi, parse_unary, {
    multi: Multiply,
    div: Divide,
    rem: Mod
});

parse_common_op!(parse_plus, parse_multi, {
    plus: Plus,
    minus: Minus
});
parse_common_op!(parse_cmp, parse_plus, {
    lt: Lt,
    gt: Gt,
    lte: Lte,
    gte: Gte
});
parse_common_op!(parse_eq, parse_cmp, {
    eq: Eq,
    ne: Ne,
    eq_full: EqFull,
    ne_full: NeFull
});
parse_common_op!(parse_bit_and, parse_eq, { bit_and: BitAnd });
parse_common_op!(parse_bit_xor, parse_bit_and, { bit_xor: BitXor });
parse_common_op!(parse_bit_or, parse_bit_xor, { bit_or: BitOr });
parse_common_op!(parse_and, parse_bit_or, { and: LogicAnd });
parse_common_op!(parse_or, parse_and, { or: LogicOr });
