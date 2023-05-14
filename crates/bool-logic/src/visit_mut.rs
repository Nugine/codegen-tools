use crate::ast::{All, Any, Expr, Not, Var};

#[allow(unused_variables)]
pub trait VisitMut<T> {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {}
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {}
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {}
    fn visit_mut_not(&mut self, Not(not): &mut Not<T>) {}
    fn visit_mut_var(&mut self, Var(var): &mut Var<T>) {}
    fn visit_mut_const(&mut self, b: &mut bool) {}
}

pub fn visit_mut_preorder<T, V>(expr: &mut Expr<T>, v: &mut V)
where
    V: VisitMut<T>,
{
    v.visit_mut_expr(expr);
    match expr {
        Expr::Any(any) => {
            v.visit_mut_any(any);
            any.0.iter_mut().for_each(|e| visit_mut_preorder(e, v))
        }
        Expr::All(all) => {
            v.visit_mut_all(all);
            all.0.iter_mut().for_each(|e| visit_mut_preorder(e, v))
        }
        Expr::Not(not) => {
            v.visit_mut_not(not);
            visit_mut_preorder(&mut *not.0, v)
        }
        Expr::Var(var) => {
            v.visit_mut_var(var);
        }
        Expr::Const(b) => {
            v.visit_mut_const(b);
        }
    }
}
