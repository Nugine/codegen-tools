use crate::ast::*;
use crate::vis::Visitor;

use std::cmp::Ordering::*;
use std::collections::BTreeSet;
use std::mem;

use itertools::Itertools;
use rust_utils::map_collect;
use rust_utils::map_collect_vec;

pub struct FlattenSingleElement;

impl Visitor for FlattenSingleElement {
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

pub struct FlattenNestedList;

#[derive(PartialEq, Eq)]
enum ListKind {
    All,
    Any,
}

impl FlattenNestedList {
    fn flatten_nested_list(v: &mut Vec<Expr>, kind: ListKind) {
        let mut i = 0;
        while i < v.len() {
            if kind == ListKind::Any {
                if let Expr::Any(Any(any)) = &mut v[i] {
                    let nested = mem::take(any);
                    v.extend(nested);
                }
            }
            if kind == ListKind::All {
                if let Expr::All(All(all)) = &mut v[i] {
                    let nested = mem::take(all);
                    v.extend(nested);
                }
            }
            i += 1;
        }
        v.retain(|x| match x {
            Expr::Any(Any(any)) => !any.is_empty(),
            Expr::All(All(all)) => !all.is_empty(),
            _ => true,
        });
    }
}

impl Visitor for FlattenNestedList {
    fn visit_all(&mut self, All(all): &mut All) {
        Self::flatten_nested_list(all, ListKind::All);
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        Self::flatten_nested_list(any, ListKind::Any);
    }
}

pub struct SimplifyAllOfAny;

impl Visitor for SimplifyAllOfAny {
    fn visit_all(&mut self, All(all): &mut All) {
        let mut i = 0;
        while i < all.len() {
            if let Expr::Any(Any(any)) = &all[i] {
                let has_same = any.iter().cartesian_product(all.iter()).any(|(x, y)| x == y);
                if has_same {
                    all.remove(i);
                    continue;
                }
            }
            i += 1;
        }
    }
}

pub struct IntersectAllAnyTargetOs;

impl IntersectAllAnyTargetOs {
    fn is_any_of_target_os(x: &Expr) -> bool {
        let Expr::Any(Any(any)) = x else { return false };
        any.iter().all(|x| matches!(x, Expr::Atom(Pred::TargetOs(_))))
    }

    fn to_target_os_set(x: &Expr) -> BTreeSet<&str> {
        let Expr::Any(Any(any)) = x else { panic!() };
        map_collect(any, |x: &Expr| {
            let Expr::Atom(Pred::TargetOs(os)) = x else { panic!() };
            os.as_str()
        })
    }
}

impl Visitor for IntersectAllAnyTargetOs {
    fn visit_expr(&mut self, x: &mut Expr) {
        let Expr::All(All(all)) = x else {return};

        if !all.iter().all(Self::is_any_of_target_os) {
            return;
        }

        let [first, xs @.. ] = all.as_slice() else {return};

        let mut final_set = Self::to_target_os_set(first);

        for x in xs {
            let os_set = Self::to_target_os_set(x);
            final_set = final_set.intersection(&os_set).cloned().collect();
        }

        *x = expr(any(map_collect_vec(final_set, |os| expr(target_os(os)))));
    }
}

pub struct ListDedup;

impl ListDedup {
    fn dedup(v: &mut Vec<Expr>) {
        let mut i = 0;
        while i < v.len() {
            let mut j = i + 1;
            while j < v.len() {
                if v[i] == v[j] {
                    v.remove(j);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }
}

impl Visitor for ListDedup {
    fn visit_all(&mut self, All(all): &mut All) {
        Self::dedup(all);
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        Self::dedup(any);
    }
}

pub struct SortTargetOs;

impl SortTargetOs {
    fn sort(es: &mut [Expr]) {
        es.sort_by(|lhs, rhs| {
            let Expr::Atom(Pred::TargetOs(lhs)) = lhs else { return Equal };
            let Expr::Atom(Pred::TargetOs(rhs)) = rhs else { return Equal };
            lhs.cmp(rhs)
        })
    }
}

impl Visitor for SortTargetOs {
    fn visit_all(&mut self, All(all): &mut All) {
        Self::sort(all)
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        Self::sort(any)
    }
}

/// Simplify `all(not(any(...)), any(...))`
pub struct SimplifyAllNotAny;

impl SimplifyAllNotAny {
    fn match_pattern(e: &mut Expr) -> Option<(&mut Vec<Expr>, &mut Vec<Expr>)> {
        if let Expr::All(All(all)) = e {
            if let [Expr::Not(Not(not)), Expr::Any(Any(pos))] = all.as_mut_slice() {
                if let Expr::Any(Any(neg)) = &mut **not {
                    return Some((neg, pos));
                }
            }
        }
        None
    }
}

impl Visitor for SimplifyAllNotAny {
    fn visit_expr(&mut self, expr: &mut Expr) {
        let Some((neg, pos)) = Self::match_pattern(expr) else {return};

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

pub struct SimplifyAnyOfAll;

impl Visitor for SimplifyAnyOfAll {
    fn visit_any(&mut self, Any(any): &mut Any) {
        let mut i = 0;
        while i < any.len() {
            if let Expr::All(All(all)) = &any[i] {
                let needs_remove = all.iter().any(|x| any.contains(x));
                if needs_remove {
                    any.remove(i);
                    continue;
                }
            }
            i += 1;
        }
    }
}
