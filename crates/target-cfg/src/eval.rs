use crate::ast::*;

pub fn simplify(expr: &mut Expr) {
    visit_preorder(expr, &mut PredListSingleElement);
}

trait Visitor {
    fn visit_expr(&mut self, _x: &mut Expr) {}

    fn visit_all(&mut self, _all: &mut All) {}

    fn visit_any(&mut self, _any: &mut Any) {}

    fn visit_not(&mut self, _not: &mut Not) {}

    fn visit_pred(&mut self, _pred: &mut Pred) {}
}

fn visit_preorder<V>(expr: &mut Expr, v: &mut V)
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

struct PredListSingleElement;

impl Visitor for PredListSingleElement {
    fn visit_expr(&mut self, x: &mut Expr) {
        let vec = match x {
            Expr::Any(Any(v)) => v,
            Expr::All(All(v)) => v,
            _ => return,
        };
        if vec.len() == 1 {
            let single = vec.pop().unwrap();
            *x = single;
        }
    }
}
