pub(crate) fn dash_to_camel(s: &str) -> String {
    let mut camel_name = String::new();
    let mut next_upper = false;
    for c in s.chars() {
        if c == '-' {
            next_upper = true;
        } else if next_upper {
            next_upper = false;
            camel_name.push(c.to_ascii_uppercase());
        } else {
            camel_name.push(c);
        }
    }
    camel_name
}

// 生成parse方法
#[macro_export]
macro_rules! parse_common_op {
  ($cur:ident, $child:ident, { $($rule:ident: $t:ident),* }) => {
      fn $cur(pair: pest::iterators::Pair<'_, Rule>) -> Box<TmplExpr> {
          let mut pairs = pair.into_inner();
          let mut ret = $child(pairs.next().unwrap());
          while let Some(op) = pairs.next() {
              let next = $child(pairs.next().unwrap());
              ret = Box::new(match op.as_rule() {
                  $(Rule::$rule => TmplExpr::$t(ret, next),)*
                  _ => unreachable!()
              });
          }
          ret
      }
  }
}
