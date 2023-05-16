use crate::ast::{All, Any, Expr, Not};
use crate::visit_mut::*;

use std::mem;
use std::ops::Not as _;

use rust_utils::default::default;

pub struct FlattenSingle;

impl<T> VisitMut<T> for FlattenSingle {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        match expr {
            Expr::Any(Any(any)) => {
                if any.is_empty() {
                    *expr = Expr::Const(false);
                } else if any.len() == 1 {
                    *expr = any.pop().unwrap();
                }
            }
            Expr::All(All(all)) => {
                if all.is_empty() {
                    *expr = Expr::Const(true);
                } else if all.len() == 1 {
                    *expr = all.pop().unwrap();
                }
            }
            _ => {}
        };

        walk_expr(self, expr)
    }
}

pub struct FlattenNestedList;

impl FlattenNestedList {
    fn flatten_any<T>(list: &mut Vec<Expr<T>>) {
        let has_any = list.iter().any(|expr| matches!(expr, Expr::Any(_)));
        if has_any.not() {
            return;
        }
        let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
        for expr in list.drain(..) {
            if let Expr::Any(Any(any)) = expr {
                ans.extend(any);
            } else {
                ans.push(expr);
            }
        }
        *list = ans;
    }

    fn flatten_all<T>(list: &mut Vec<Expr<T>>) {
        let has_all = list.iter().any(|expr| matches!(expr, Expr::All(_)));
        if has_all.not() {
            return;
        }
        let mut ans: Vec<Expr<T>> = Vec::with_capacity(list.len());
        for expr in list.drain(..) {
            if let Expr::All(All(all)) = expr {
                ans.extend(all);
            } else {
                ans.push(expr);
            }
        }
        *list = ans;
    }
}

impl<T> VisitMut<T> for FlattenNestedList {
    fn visit_mut_any(&mut self, Any(list): &mut Any<T>) {
        Self::flatten_any(list);
        walk_expr_list(self, list);
    }

    fn visit_mut_all(&mut self, All(list): &mut All<T>) {
        Self::flatten_all(list);
        walk_expr_list(self, list);
    }
}

pub struct DedupList;

impl<T> VisitMut<T> for DedupList
where
    T: Eq,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        if let Some(list) = expr.as_expr_list_mut() {
            let mut i = 0;
            while i < list.len() {
                let mut j = i + 1;
                while j < list.len() {
                    if list[i] == list[j] {
                        list.remove(j);
                    } else {
                        j += 1;
                    }
                }
                i += 1;
            }
        }
        walk_expr(self, expr);
    }
}

pub struct EvalConst;

impl EvalConst {
    fn eval_any<T>(any: &[Expr<T>]) -> Option<bool> {
        if any.iter().any(|expr| matches!(expr, Expr::Const(true))) {
            return Some(true);
        }
        any.iter().try_fold(false, |acc, x| match x {
            Expr::Const(val) => Some(acc | val),
            _ => None,
        })
    }

    fn eval_all<T>(all: &[Expr<T>]) -> Option<bool> {
        if all.iter().any(|expr| matches!(expr, Expr::Const(false))) {
            return Some(false);
        }
        all.iter().try_fold(true, |acc, x| match x {
            Expr::Const(val) => Some(acc & val),
            _ => None,
        })
    }

    fn eval_not<T>(not: &Expr<T>) -> Option<bool> {
        if let Expr::Const(val) = not {
            return Some(val.not());
        }
        None
    }
}

impl<T> VisitMut<T> for EvalConst {
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        walk_expr(self, expr);

        match expr {
            Expr::Any(Any(any)) => {
                if let Some(val) = Self::eval_any(any) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::All(All(all)) => {
                if let Some(val) = Self::eval_all(all) {
                    *expr = Expr::Const(val);
                }
            }
            Expr::Not(Not(not)) => {
                if let Some(val) = Self::eval_not(not) {
                    *expr = Expr::Const(val);
                }
            }
            _ => {}
        }
    }
}

pub struct SimplifyNestedList;

impl SimplifyNestedList {
    fn contains_cross_same<T: Eq>(lhs: &[T], rhs: &[T]) -> bool {
        lhs.iter().any(|x| rhs.contains(x))
    }
}

impl<T> VisitMut<T> for SimplifyNestedList
where
    T: Eq,
{
    /// `any(x0, all(x0, x1), x2) => any(x0, x2)`
    fn visit_mut_any(&mut self, Any(any): &mut Any<T>) {
        let mut i = 0;
        while i < any.len() {
            if let Expr::All(All(all)) = &any[i] {
                if Self::contains_cross_same(all, any) {
                    any.remove(i);
                    continue;
                }
            }

            i += 1;
        }

        walk_expr_list(self, any);
    }

    /// `all(x0, any(x0, x1), x2) => all(x0, x2)`
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        let mut i = 0;
        while i < all.len() {
            if let Expr::Any(Any(any)) = &all[i] {
                if Self::contains_cross_same(any, all) {
                    all.remove(i);
                    continue;
                }
            }

            i += 1;
        }

        walk_expr_list(self, all);
    }
}

/// Simplify `all(not(any(...)), any(...))`
pub struct SimplifyAllNotAny;

impl SimplifyAllNotAny {
    fn counteract<T>(neg: &[Expr<T>], pos: &mut Vec<Expr<T>>)
    where
        T: Eq,
    {
        let mut i = 0;
        while i < pos.len() {
            if neg.contains(&pos[i]) {
                pos.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

impl<T> VisitMut<T> for SimplifyAllNotAny
where
    T: Eq,
{
    fn visit_mut_all(&mut self, All(all): &mut All<T>) {
        if let [Expr::Not(Not(not)), Expr::Any(Any(pos))] = all.as_mut_slice() {
            if let Expr::Any(Any(neg)) = &mut **not {
                Self::counteract(neg, pos);
            }
        }
        if let [Expr::Any(Any(pos)), Expr::Not(Not(not))] = all.as_mut_slice() {
            if let Expr::Any(Any(neg)) = &mut **not {
                Self::counteract(neg, pos);
            }
        }

        walk_expr_list(self, all);
    }
}

/// Simplify `all(any(...), any(...))`
pub struct SimplifyAllOfAny;

impl SimplifyAllOfAny {
    fn intersect<T: Eq>(lhs: &mut Vec<Expr<T>>, rhs: &mut Vec<Expr<T>>, ans: &mut Vec<Expr<T>>) {
        ans.clear();
        for x in lhs.drain(..) {
            if rhs.contains(&x) {
                ans.push(x);
            }
        }
        rhs.clear();
    }

    fn simplify<T: Eq>(expr: &mut Expr<T>) {
        let Expr::All(All(all)) = expr else {return};

        let is_all_of_any = all.iter().all(|expr| matches!(expr, Expr::Any(_)));
        if is_all_of_any.not() {
            return;
        }

        let [first,  others @ ..] = all.as_mut_slice() else { return };
        let Expr::Any(Any(first)) = first else { panic!() };

        let mut buf: Vec<Expr<T>> = default();

        for x in others {
            let Expr::Any(Any(any)) = x else { panic!() };
            Self::intersect(first, any, &mut buf);
            mem::swap(first, &mut buf);
        }

        *expr = Expr::Any(Any(mem::take(first)));
    }
}

impl<T> VisitMut<T> for SimplifyAllOfAny
where
    T: Eq,
{
    fn visit_mut_expr(&mut self, expr: &mut Expr<T>) {
        Self::simplify(expr);
        walk_expr(self, expr);
    }
}
