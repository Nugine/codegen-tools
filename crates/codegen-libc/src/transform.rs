use codegen_cfg::ast::{flag, key_value, Expr, Not, Pred, Var};
use codegen_cfg::bool_logic::transform::*;
use codegen_cfg::bool_logic::visit_mut::*;

use std::cmp::Ordering::{self, *};

use log::debug;

pub fn simplified_expr(x: impl Into<Expr>) -> Expr {
    let mut x = x.into();

    debug!("input: {x}");

    UnifyTargetFamily.visit_mut_expr(&mut x);

    for _ in 0..3 {
        // debug!("before FlattenSingle: {x}");
        FlattenSingle.visit_mut_expr(&mut x);

        // debug!("before FlattenNestedList: {x}");
        FlattenNestedList.visit_mut_expr(&mut x);

        // debug!("before FlattenByDistributive: {x}");
        DedupList.visit_mut_expr(&mut x);

        // debug!("before EvalConst: {x}");
        EvalConst.visit_mut_expr(&mut x);

        // debug!("before SimplifyNestedList: {x}");
        SimplifyNestedList.visit_mut_expr(&mut x);

        // debug!("before MergeAllOfNotAny: {x}");
        MergeAllOfNotAny.visit_mut_expr(&mut x);

        // debug!("before SimplifyAllNotAny: {x}");
        SimplifyAllNotAny.visit_mut_expr(&mut x);

        // debug!("before EvalConst: {x}");
        EvalConst.visit_mut_expr(&mut x);
    }

    SimplifyTargetFamily.visit_mut_expr(&mut x);

    // debug!("before SortByPriority: {x}");
    SortByPriority.visit_mut_expr(&mut x);

    // debug!("before SortByValue: {x}");
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
                _ => 0,
            },
            Expr::Const(_) => panic!(),
        })
    }
}

impl VisitMut<Pred> for SortByPriority {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_mut_expr_list() {
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

impl SortByValue {
    fn cmp_var(lhs: &Expr, rhs: &Expr) -> Ordering {
        let Expr::Var(Var(lhs)) = lhs else { return Equal };
        let Expr::Var(Var(rhs)) = rhs else { return Equal };

        let ok = Ord::cmp(lhs.key.as_str(), rhs.key.as_str());

        match (lhs.value.as_deref(), rhs.value.as_deref()) {
            (None, None) => ok,
            (Some(lv), Some(rv)) => ok.then_with(|| Ord::cmp(lv, rv)),
            (None, Some(_)) => Less,
            (Some(_), None) => Greater,
        }
    }

    fn cmp_not(lhs: &Expr, rhs: &Expr) -> Ordering {
        let Expr::Not(Not(lhs)) = lhs else { return Equal };
        let Expr::Not(Not(rhs)) = rhs else { return Equal };

        Self::cmp_var(lhs, rhs)
    }
}

impl VisitMut<Pred> for SortByValue {
    fn visit_mut_expr(&mut self, expr: &mut Expr) {
        if let Some(list) = expr.as_mut_expr_list() {
            list.sort_by(Self::cmp_var);
            list.sort_by(Self::cmp_not);
        }

        walk_mut_expr(self, expr);
    }
}

struct UnifyTargetFamily;

impl VisitMut<Pred> for UnifyTargetFamily {
    fn visit_mut_var(&mut self, Var(pred): &mut Var<Pred>) {
        if pred.value.is_none() && matches!(pred.key.as_str(), "unix" | "windows" | "wasm") {
            *pred = key_value("target_family", pred.key.as_str());
        }
    }
}

struct SimplifyTargetFamily;

impl VisitMut<Pred> for SimplifyTargetFamily {
    fn visit_mut_var(&mut self, Var(pred): &mut Var<Pred>) {
        if pred.key == "target_family" {
            if let Some(value) = pred.value.as_deref() {
                if matches!(value, "unix" | "windows" | "wasm") {
                    *pred = flag(value);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use codegen_cfg::ast::*;

    #[test]
    fn sort() {
        let mut expr = expr(all((not(flag("unix")), flag("unix"))));
        SortByPriority.visit_mut_expr(&mut expr);
        assert_eq!(expr.to_string(), "all(unix, not(unix))");
    }
}
