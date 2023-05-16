use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr<T> {
    Any(Any<T>),
    All(All<T>),
    Not(Not<T>),
    Var(Var<T>),
    Const(bool),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Any<T>(pub Vec<Expr<T>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct All<T>(pub Vec<Expr<T>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Not<T>(pub Box<Expr<T>>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Var<T>(pub T);

pub fn expr<T>(x: impl Into<Expr<T>>) -> Expr<T> {
    x.into()
}

pub fn any<T>(x: impl Into<Any<T>>) -> Any<T> {
    x.into()
}

pub fn all<T>(x: impl Into<All<T>>) -> All<T> {
    x.into()
}

pub fn not<T>(x: impl Into<Not<T>>) -> Not<T> {
    x.into()
}

pub fn var<T>(x: impl Into<Var<T>>) -> Var<T> {
    x.into()
}

pub fn const_<T>(x: bool) -> Expr<T> {
    x.into()
}

impl<T> From<Any<T>> for Expr<T> {
    fn from(any: Any<T>) -> Self {
        Expr::Any(any)
    }
}

impl<T> From<All<T>> for Expr<T> {
    fn from(all: All<T>) -> Self {
        Expr::All(all)
    }
}

impl<T> From<Not<T>> for Expr<T> {
    fn from(not: Not<T>) -> Self {
        Expr::Not(not)
    }
}

impl<T> From<Var<T>> for Expr<T> {
    fn from(var: Var<T>) -> Self {
        Expr::Var(var)
    }
}

impl<T> From<bool> for Expr<T> {
    fn from(b: bool) -> Self {
        Expr::Const(b)
    }
}

impl<T> From<Vec<Expr<T>>> for Any<T> {
    fn from(exprs: Vec<Expr<T>>) -> Self {
        Any(exprs)
    }
}

impl<T> From<Vec<Expr<T>>> for All<T> {
    fn from(exprs: Vec<Expr<T>>) -> Self {
        All(exprs)
    }
}

impl<T> From<Box<Expr<T>>> for Not<T> {
    fn from(expr: Box<Expr<T>>) -> Self {
        Not(expr)
    }
}

impl<T> From<Expr<T>> for Not<T> {
    fn from(expr: Expr<T>) -> Self {
        Not(Box::new(expr))
    }
}

impl<T> From<Any<T>> for Not<T> {
    fn from(any: Any<T>) -> Self {
        Not(Box::new(any.into()))
    }
}

impl<T> From<All<T>> for Not<T> {
    fn from(all: All<T>) -> Self {
        Not(Box::new(all.into()))
    }
}

impl<T> From<Var<T>> for Not<T> {
    fn from(var: Var<T>) -> Self {
        Not(Box::new(var.into()))
    }
}

impl<T> From<T> for Not<T> {
    fn from(var: T) -> Self {
        Self::from(Var(var))
    }
}

impl<T> From<T> for Var<T> {
    fn from(var: T) -> Self {
        Var(var)
    }
}

macro_rules! impl_from_tuple {
    ($ty:ident, ()) => {
        impl_from_tuple!(@expand $ty, ());
    };
    ($ty:ident, ($x:tt, $($xs:tt,)*)) => {
        impl_from_tuple!($ty, ($($xs,)*));
        impl_from_tuple!(@expand $ty, ($x, $($xs,)*));
    };
    (@expand $ty:ident, ($($tt:tt,)*)) => {
        #[doc(hidden)] // too ugly
        #[allow(non_camel_case_types)]
        impl<T, $($tt),*> From<($($tt,)*)>  for $ty<T>
        where
            $($tt: Into<Expr<T>>,)*
        {
            fn from(($($tt,)*): ($($tt,)*)) -> Self {
                Self::from(vec![$(Into::into($tt)),*])
            }
        }
    };
}

impl_from_tuple!(
    Any,
    (
        x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, //
        x12, x13, x14, x15, x16, x17, x18, x19, x20, x21, x22, x23, //
        x24, x25, x26, x27, x28, x29, x30, x31, x32, x33, x34, x35,
    )
);

impl_from_tuple!(
    All,
    (
        x0, x1, x2, x3, x4, x5, x6, x7, x8, x9, x10, x11, //
        x12, x13, x14, x15, x16, x17, x18, x19, x20, x21, x22, x23, //
        x24, x25, x26, x27, x28, x29, x30, x31, x32, x33, x34, x35,
    )
);

impl<T> fmt::Display for Expr<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Any(Any(any)) => fmt_list(f, "any", any),
            Expr::All(All(all)) => fmt_list(f, "all", all),
            Expr::Not(Not(not)) => write!(f, "not({not})"),
            Expr::Var(Var(x)) => write!(f, "{x}"),
            Expr::Const(b) => write!(f, "{b}"),
        }
    }
}

fn fmt_list<T>(f: &mut fmt::Formatter<'_>, name: &str, list: &[Expr<T>]) -> fmt::Result
where
    T: fmt::Display,
{
    write!(f, "{name}(")?;
    for (i, e) in list.iter().enumerate() {
        if i != 0 {
            write!(f, ", ")?;
        }
        write!(f, "{e}")?;
    }
    write!(f, ")")
}

impl<T> Expr<T> {
    pub fn as_expr_list_mut(&mut self) -> Option<&mut Vec<Expr<T>>> {
        match self {
            Expr::Any(Any(list)) | Expr::All(All(list)) => Some(list),
            _ => None,
        }
    }
}
