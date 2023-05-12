use codegen_cfg::ast::{All, Any, Expr, Pred};
use codegen_cfg::transform::*;
use codegen_cfg::vis::{visit_preorder, Visitor};

pub fn simplified_expr(x: impl Into<Expr>) -> Expr {
    let mut x = x.into();

    visit_preorder(&mut x, &mut FlattenSingleElement);
    visit_preorder(&mut x, &mut FlattenNestedList);
    visit_preorder(&mut x, &mut ListDedup);
    visit_preorder(&mut x, &mut SimplifyAllOfAny);
    visit_preorder(&mut x, &mut FlattenSingleElement);
    visit_preorder(&mut x, &mut IntersectAllAnyTargetOs);
    visit_preorder(&mut x, &mut FlattenNestedList);
    visit_preorder(&mut x, &mut ListDedup);

    visit_preorder(&mut x, &mut SortByPriority);
    visit_preorder(&mut x, &mut SortTargetOs);

    visit_preorder(&mut x, &mut CounteractAllNotAny);

    x
}

struct SortByPriority;

impl SortByPriority {
    fn stable_sort(es: &mut [Expr]) {
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
                _ => unimplemented!(),
            },
        })
    }
}

impl Visitor for SortByPriority {
    fn visit_all(&mut self, All(all): &mut All) {
        Self::stable_sort(all)
    }

    fn visit_any(&mut self, Any(any): &mut Any) {
        Self::stable_sort(any)
    }
}
