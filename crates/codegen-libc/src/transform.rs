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
    visit_mut_preorder(&mut x, &mut SortTargetOs);

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
            Expr::Var(Var(pred)) => match pred {
                Pred::TargetFamily(_) => 1,
                Pred::TargetArch(_) => 2,
                Pred::TargetVendor(_) => 3,
                Pred::TargetOs(_) => 4,
                Pred::TargetEnv(_) => 5,
                Pred::TargetPointerWidth(_) => 6,
                _ => unimplemented!(),
            },
            Expr::Const(_) => panic!(),
        })
    }
}

struct SortTargetOs;

impl VisitMut<Pred> for SortTargetOs {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        let list = match expr {
            Expr::Any(Any(any)) => any,
            Expr::All(All(all)) => all,
            _ => return,
        };

        list.sort_by(|lhs, rhs| {
            let Expr::Var(Var(Pred::TargetOs(lhs))) = lhs else { return Equal };
            let Expr::Var(Var(Pred::TargetOs(rhs))) = rhs else { return Equal };
            lhs.cmp(rhs)
        })
    }
}
