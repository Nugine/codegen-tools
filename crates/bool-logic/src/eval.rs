use crate::ast::{All, Any, Expr, Not, Var};

pub fn eval_with<T>(expr: &Expr<T>, f: &impl Fn(&T) -> bool) -> bool {
    match expr {
        Expr::Any(Any(list)) => list.iter().any(|e| eval_with(e, f)),
        Expr::All(All(list)) => list.iter().all(|e| eval_with(e, f)),
        Expr::Not(Not(not)) => !eval_with(not, f),
        Expr::Var(Var(var)) => f(var),
        Expr::Const(b) => *b,
    }
}
