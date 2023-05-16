use codegen_cfg::ast::{Expr, Pred, Var};
use codegen_cfg::bool_logic::transform::*;
use codegen_cfg::bool_logic::visit_mut::*;

use std::cmp::Ordering::*;

use log::debug;

pub fn simplified_expr(x: impl Into<Expr>) -> Expr {
    let mut x = x.into();

    debug!("input: {x}");

    for _ in 0..3 {
        debug!("before FlattenSingle: {x}");
        FlattenSingle.visit_mut_expr(&mut x);

        debug!("before FlattenNestedList: {x}");
        FlattenNestedList.visit_mut_expr(&mut x);

        debug!("before FlattenByDeMorgan: {x}");
        FlattenByDeMorgan.visit_mut_expr(&mut x);

        debug!("before FlattenByDistributive: {x}");
        DedupList.visit_mut_expr(&mut x);

        debug!("before EvalConst: {x}");
        EvalConst.visit_mut_expr(&mut x);

        debug!("before SimplifyNestedList: {x}");
        SimplifyNestedList.visit_mut_expr(&mut x);

        debug!("before SimplifyAllNotAny: {x}");
        SimplifyAllNotAny.visit_mut_expr(&mut x);

        debug!("before EvalConst: {x}");
        EvalConst.visit_mut_expr(&mut x);
    }

    debug!("before SortByPriority: {x}");
    SortByPriority.visit_mut_expr(&mut x);

    debug!("before SortByValue: {x}");
    SortByValue.visit_mut_expr(&mut x);

    debug!("output: {x}");

    x
}

struct SortByPriority;

impl SortByPriority {
    fn get_priority(x: &Expr) -> Option<u32> {
        Some(match x {
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
                _ => return None,
            },
            Expr::Const(_) => panic!(),
        })
    }
}

impl VisitMut<Pred> for SortByPriority {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_expr_list_mut() {
            list.sort_by(|lhs, rhs| {
                let Some(lhs) = Self::get_priority(lhs) else {return Equal};
                let Some(rhs) = Self::get_priority(rhs) else {return Equal};
                lhs.cmp(&rhs)
            })
        }

        walk_mut_expr(self, expr);
    }
}

struct SortByValue;

impl VisitMut<Pred> for SortByValue {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_expr_list_mut() {
            list.sort_by(|lhs, rhs| {
                let Expr::Var(Var(lhs)) = lhs else { return Equal };
                let Expr::Var(Var(rhs)) = rhs else { return Equal };
                if lhs.key != rhs.key {
                    return Equal;
                }

                let Some(lhs) = lhs.value.as_deref() else { return Equal };
                let Some(rhs) = rhs.value.as_deref() else { return Equal };
                lhs.cmp(rhs)
            });
        }

        walk_mut_expr(self, expr);
    }
}
