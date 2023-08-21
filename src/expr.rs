use crate::binding_map::{BindingMapCollector, BindingMapKeys};

#[derive(Debug)]
pub(crate) enum TmplExpr {
    ScopeIndex(usize),
    Ident(String),
    ToStringWithoutUndefined(Box<TmplExpr>),

    LitUndefined,
    LitNull,
    LitStr(String),
    LitInt(i32),
    LitFloat(f64),
    LitBool(bool),
    LitObj(Vec<(Option<String>, TmplExpr)>),
    LitArr(Vec<TmplExpr>),

    StaticMember(Box<TmplExpr>, String),
    DynamicMember(Box<TmplExpr>, Box<TmplExpr>),
    FuncCall(Box<TmplExpr>, Vec<TmplExpr>),

    Reverse(Box<TmplExpr>),
    BitReverse(Box<TmplExpr>),
    Positive(Box<TmplExpr>),
    Negative(Box<TmplExpr>),

    Multiply(Box<TmplExpr>, Box<TmplExpr>),
    Divide(Box<TmplExpr>, Box<TmplExpr>),
    Mod(Box<TmplExpr>, Box<TmplExpr>),
    Plus(Box<TmplExpr>, Box<TmplExpr>),
    Minus(Box<TmplExpr>, Box<TmplExpr>),

    Lt(Box<TmplExpr>, Box<TmplExpr>),
    Gt(Box<TmplExpr>, Box<TmplExpr>),
    Lte(Box<TmplExpr>, Box<TmplExpr>),
    Gte(Box<TmplExpr>, Box<TmplExpr>),
    Eq(Box<TmplExpr>, Box<TmplExpr>),
    Ne(Box<TmplExpr>, Box<TmplExpr>),
    EqFull(Box<TmplExpr>, Box<TmplExpr>),
    NeFull(Box<TmplExpr>, Box<TmplExpr>),

    BitAnd(Box<TmplExpr>, Box<TmplExpr>),
    BitXor(Box<TmplExpr>, Box<TmplExpr>),
    BitOr(Box<TmplExpr>, Box<TmplExpr>),
    LogicAnd(Box<TmplExpr>, Box<TmplExpr>),
    LogicOr(Box<TmplExpr>, Box<TmplExpr>),

    Cond(Box<TmplExpr>, Box<TmplExpr>, Box<TmplExpr>),
}

#[derive(Debug, PartialEq, PartialOrd)]
pub(crate) enum TmplExprLevel {
    Lit = 0,
    Member = 1,
    Unary = 2,
    Multiply = 3,
    Plus = 4,
    Comparison = 5,
    Eq = 6,
    BitAnd = 7,
    BitXor = 8,
    BitOr = 9,
    LogicAnd = 10,
    LogicOr = 11,
    Cond = 12,
    Comma = 13,
}

impl TmplExpr {
    pub(crate) fn level(&self) -> TmplExprLevel {
        match self {
            TmplExpr::ScopeIndex(_) => TmplExprLevel::Member,
            TmplExpr::Ident(_) => TmplExprLevel::Lit,
            TmplExpr::ToStringWithoutUndefined(_) => TmplExprLevel::Member,
            TmplExpr::LitUndefined => TmplExprLevel::Lit,
            TmplExpr::LitNull => TmplExprLevel::Lit,
            TmplExpr::LitStr(_) => TmplExprLevel::Lit,
            TmplExpr::LitInt(_) => TmplExprLevel::Lit,
            TmplExpr::LitFloat(_) => TmplExprLevel::Lit,
            TmplExpr::LitBool(_) => TmplExprLevel::Lit,
            TmplExpr::LitObj(_) => TmplExprLevel::Lit,
            TmplExpr::LitArr(_) => TmplExprLevel::Lit,
            TmplExpr::StaticMember(_, _) => TmplExprLevel::Member,
            TmplExpr::DynamicMember(_, _) => TmplExprLevel::Member,
            TmplExpr::FuncCall(_, _) => TmplExprLevel::Member,
            TmplExpr::Reverse(_) => TmplExprLevel::Unary,
            TmplExpr::BitReverse(_) => TmplExprLevel::Unary,
            TmplExpr::Positive(_) => TmplExprLevel::Unary,
            TmplExpr::Negative(_) => TmplExprLevel::Unary,
            TmplExpr::Multiply(_, _) => TmplExprLevel::Multiply,
            TmplExpr::Divide(_, _) => TmplExprLevel::Multiply,
            TmplExpr::Mod(_, _) => TmplExprLevel::Multiply,
            TmplExpr::Plus(_, _) => TmplExprLevel::Plus,
            TmplExpr::Minus(_, _) => TmplExprLevel::Plus,
            TmplExpr::Lt(_, _) => TmplExprLevel::Comparison,
            TmplExpr::Gt(_, _) => TmplExprLevel::Comparison,
            TmplExpr::Lte(_, _) => TmplExprLevel::Comparison,
            TmplExpr::Gte(_, _) => TmplExprLevel::Comparison,
            TmplExpr::Eq(_, _) => TmplExprLevel::Eq,
            TmplExpr::Ne(_, _) => TmplExprLevel::Eq,
            TmplExpr::EqFull(_, _) => TmplExprLevel::Eq,
            TmplExpr::NeFull(_, _) => TmplExprLevel::Eq,
            TmplExpr::BitAnd(_, _) => TmplExprLevel::BitAnd,
            TmplExpr::BitXor(_, _) => TmplExprLevel::BitXor,
            TmplExpr::BitOr(_, _) => TmplExprLevel::BitOr,
            TmplExpr::LogicAnd(_, _) => TmplExprLevel::LogicAnd,
            TmplExpr::LogicOr(_, _) => TmplExprLevel::LogicOr,
            TmplExpr::Cond(_, _, _) => TmplExprLevel::Cond,
        }
    }

    // this function finds which keys can be put into the binding map,
    // and convert scope names to scope indexes at the same time.
    pub(crate) fn get_binding_map_keys(
        &mut self,
        bmc: &mut BindingMapCollector,
        scope_names: &Vec<String>,
        should_disable: bool,
    ) -> Option<BindingMapKeys> {
        let mut bmk = BindingMapKeys::new();
        self.get_binding_map_keys_rec(bmc, scope_names, should_disable, &mut bmk);
        if should_disable {
            None
        } else {
            Some(bmk)
        }
    }

    fn get_binding_map_keys_rec(
        &mut self,
        bmc: &mut BindingMapCollector,
        scope_names: &Vec<String>,
        should_disable: bool,
        bmk: &mut BindingMapKeys,
    ) {
        match self {
            TmplExpr::ScopeIndex(_) => {}
            TmplExpr::Ident(x) => {
                if let Some(n) = scope_names.iter().rposition(|n| n == x) {
                    *self = TmplExpr::ScopeIndex(n);
                } else if should_disable {
                    bmc.disable_field(x);
                } else if let Some(index) = bmc.add_field(x) {
                    bmk.add(x, index);
                }
            }
            TmplExpr::ToStringWithoutUndefined(x) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }

            TmplExpr::LitUndefined => {}
            TmplExpr::LitNull => {}
            TmplExpr::LitStr(_) => {}
            TmplExpr::LitInt(_) => {}
            TmplExpr::LitFloat(_) => {}
            TmplExpr::LitBool(_) => {}
            TmplExpr::LitObj(x) => {
                for x in x.iter_mut() {
                    x.1.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                }
            }
            TmplExpr::LitArr(x) => {
                for x in x.iter_mut() {
                    x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                }
            }

            TmplExpr::StaticMember(x, _) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::DynamicMember(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::FuncCall(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                for y in y.iter_mut() {
                    y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                }
            }

            TmplExpr::Reverse(x) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::BitReverse(x) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Positive(x) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Negative(x) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }

            TmplExpr::Multiply(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Divide(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Mod(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Plus(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Minus(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }

            TmplExpr::Lt(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Gt(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Lte(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Gte(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Eq(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::Ne(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::EqFull(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::NeFull(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }

            TmplExpr::BitAnd(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::BitXor(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::BitOr(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::LogicAnd(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
            TmplExpr::LogicOr(x, y) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }

            TmplExpr::Cond(x, y, z) => {
                x.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                y.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
                z.get_binding_map_keys_rec(bmc, scope_names, should_disable, bmk);
            }
        };
    }
}
