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


impl TmplExpr {
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
