use crate::ast::{All, Any, Expr, Not, Pred};

#[allow(unused_variables)]
pub trait Visitor {
    #[inline]
    fn visit_expr(&mut self, expr: &mut Expr) {}

    #[inline]
    fn visit_any(&mut self, Any(any): &mut Any) {}

    #[inline]
    fn visit_all(&mut self, All(all): &mut All) {}

    #[inline]
    fn visit_not(&mut self, Not(not): &mut Not) {}

    #[inline]
    fn visit_pred(&mut self, pred: &mut Pred) {}
}

pub fn visit_preorder<V>(expr: &mut Expr, v: &mut V)
where
    V: Visitor,
{
    v.visit_expr(expr);

    match expr {
        Expr::Any(any) => {
            v.visit_any(any);
            for expr in &mut any.0 {
                visit_preorder(expr, v);
            }
        }
        Expr::All(all) => {
            v.visit_all(all);
            for expr in &mut all.0 {
                visit_preorder(expr, v);
            }
        }
        Expr::Not(not) => {
            v.visit_not(not);
            visit_preorder(&mut not.0, v);
        }
        Expr::Atom(pred) => {
            v.visit_pred(pred);
        }
    }
}
