use crate::ast::*;
use crate::utils::*;

use std::cmp::Ordering::*;
use std::collections::BTreeSet;

use itertools::Itertools;

pub fn simplified_cfg(x: impl Into<Cfg>) -> Cfg {
    let mut x = x.into();
    simplify(&mut x.0);
    x
}

pub fn simplify(expr: &mut Expr) {
    visit_preorder(expr, &mut FlattenSingleElement);
    visit_preorder(expr, &mut FlattenNestedPredList);
    visit_preorder(expr, &mut SimplifyAllOfAny);
    visit_preorder(expr, &mut FlattenSingleElement);
    visit_preorder(expr, &mut SortByPriority);
    visit_preorder(expr, &mut SortTargetOs);
    visit_preorder(expr, &mut IntersectAllAnyTargetOs);
    visit_preorder(expr, &mut FlattenNestedPredList);
}

trait Visitor {
    fn visit_expr(&mut self, _x: &mut Expr) {}

    fn visit_all(&mut self, All(_all): &mut All) {}

    fn visit_any(&mut self, Any(_any): &mut Any) {}

    fn visit_not(&mut self, Not(_not): &mut Not) {}

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

struct FlattenSingleElement;

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

struct FlattenNestedPredList;

impl Visitor for FlattenNestedPredList {
    fn visit_all(&mut self, All(all): &mut All) {
        let mut buf = Vec::with_capacity(all.len());
        for x in all.drain(..) {
            if let Expr::All(All(v)) = x {
                buf.extend(v);
            } else {
                buf.push(x);
            }
        }
        *all = buf
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        let mut buf = Vec::with_capacity(any.len());
        for x in any.drain(..) {
            if let Expr::Any(Any(v)) = x {
                buf.extend(v);
            } else {
                buf.push(x);
            }
        }
        *any = buf
    }
}

struct SimplifyAllOfAny;

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

struct SortByPriority;

impl SortByPriority {
    fn sort(es: &mut [Expr]) {
        es.sort_by_key(|x| match x {
            Expr::Not(_) => 101,
            Expr::Any(_) => 102,
            Expr::All(_) => 103,
            Expr::Atom(pred) => match pred {
                Pred::TargetFamily(_) => 1,
                Pred::TargetArch(_) => 2,
                Pred::TargetVendor(_) => 3,
                Pred::TargetOs(_) => 4,
                Pred::TargetEnv(_) => 5,
                Pred::TargetPointerWidth(_) => 6,
            },
        })
    }
}

impl Visitor for SortByPriority {
    fn visit_all(&mut self, All(all): &mut All) {
        Self::sort(all)
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        Self::sort(any)
    }
}

struct SortTargetOs;

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

struct IntersectAllAnyTargetOs;

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
