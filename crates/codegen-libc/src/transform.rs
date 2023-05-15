use std::cmp::Ordering::*;

use codegen_cfg::ast::{All, Any, Expr, Pred, Var};
use codegen_cfg::bool_logic::transform::*;
use codegen_cfg::bool_logic::visit_mut::{visit_mut_preorder, VisitMut};

pub fn simplified_expr(x: impl Into<Expr>) -> Expr {
    let mut x = x.into();

    for _ in 0..2 {
        visit_mut_preorder(&mut x, &mut FlattenSingle);
        visit_mut_preorder(&mut x, &mut FlattenNestedList);
        visit_mut_preorder(&mut x, &mut DedupList);

        visit_mut_preorder(&mut x, &mut SimplifyNestedList);
        visit_mut_preorder(&mut x, &mut SimplifyAllNotAny);
        visit_mut_preorder(&mut x, &mut SimplifyAllOfAny);

        visit_mut_preorder(&mut x, &mut EvalConst);
    }

    visit_mut_preorder(&mut x, &mut SortByPriority);
    visit_mut_preorder(&mut x, &mut SortByValue);

    x
}

struct SortByPriority;

impl VisitMut<Pred> for SortByPriority {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let list = match expr {
            Expr::Any(Any(any)) => any,
            Expr::All(All(all)) => all,
            _ => return,
        };

        list.sort_by_key(|x| match x {
            Expr::Not(_) => 101,
            Expr::Any(_) => 102,
            Expr::All(_) => 103,
            Expr::Var(Var(pred)) => match pred.key.as_str() {
                "target_family" => 1,
                "target_arch" => 2,
                "target_vendor" => 3,
                "target_os" => 4,
                "target_env" => 5,
                "target_pointer_width" => 6,
                _ => unimplemented!(),
            },
            Expr::Const(_) => panic!(),
        })
    }
}

struct SortByValue;

impl VisitMut<Pred> for SortByValue {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let list = match expr {
            Expr::Any(Any(any)) => any,
            Expr::All(All(all)) => all,
            _ => return,
        };

        list.sort_by(|lhs, rhs| {
            let Expr::Var(Var(lhs)) = lhs else { return Equal };
            let Expr::Var(Var(rhs)) = rhs else { return Equal };
            if lhs.key != rhs.key {
                return Equal;
            }

            let Some(lhs) = lhs.value.as_deref() else { return Equal };
            let Some(rhs) = rhs.value.as_deref() else { return Equal };
            lhs.cmp(rhs)
        })
    }
}
