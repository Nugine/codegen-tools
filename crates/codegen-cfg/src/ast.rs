use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Any(Any),
    All(All),
    Not(Not),
    Atom(Pred),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Any(pub Vec<Expr>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct All(pub Vec<Expr>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Not(pub Box<Expr>);

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Pred {
    TargetFamily(String),
    TargetVendor(String),
    TargetArch(String),
    TargetOs(String),
    TargetEnv(String),
    TargetPointerWidth(String),
}

impl From<Pred> for Expr {
    fn from(x: Pred) -> Self {
        Expr::Atom(x)
    }
}

impl From<Not> for Expr {
    fn from(x: Not) -> Self {
        Expr::Not(x)
    }
}

impl From<Any> for Expr {
    fn from(x: Any) -> Self {
        Expr::Any(x)
    }
}

impl From<All> for Expr {
    fn from(x: All) -> Self {
        Expr::All(x)
    }
}

impl From<Vec<Expr>> for Any {
    fn from(x: Vec<Expr>) -> Self {
        Any(x)
    }
}

impl From<Vec<Expr>> for All {
    fn from(x: Vec<Expr>) -> Self {
        All(x)
    }
}

impl From<Box<Expr>> for Not {
    fn from(x: Box<Expr>) -> Self {
        Not(x)
    }
}

impl From<Expr> for Not {
    fn from(x: Expr) -> Self {
        Not(Box::new(x))
    }
}

macro_rules! impl_from_tuple {
    ($ty:ty, ($tt:tt,)) => {
        impl_from_tuple!(@expand $ty, ($tt,));
    };
    ($ty:ty, ($x:tt, $($xs:tt,)+)) => {
        impl_from_tuple!($ty, ($($xs,)+));
        impl_from_tuple!(@expand $ty, ($x, $($xs,)+));
    };
    (@expand $ty:ty, ($($tt:tt,)+)) => {
        #[doc(hidden)] // too ugly
        #[allow(non_camel_case_types)]
        impl<$($tt),+> From<($($tt,)+)>  for $ty
        where
            $($tt: Into<Expr>,)+
        {
            fn from(($($tt,)+): ($($tt,)+)) -> Self {
                Self::from(vec![$(Into::into($tt)),+])
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

pub fn expr(x: impl Into<Expr>) -> Expr {
    x.into()
}

pub fn any(x: impl Into<Any>) -> Any {
    x.into()
}

pub fn all(x: impl Into<All>) -> All {
    x.into()
}

pub fn not(x: impl Into<Not>) -> Not {
    x.into()
}

pub fn target_family(s: impl Into<String>) -> Pred {
    Pred::TargetFamily(s.into())
}

pub fn target_vendor(s: impl Into<String>) -> Pred {
    Pred::TargetVendor(s.into())
}

pub fn target_arch(s: impl Into<String>) -> Pred {
    Pred::TargetArch(s.into())
}

pub fn target_os(s: impl Into<String>) -> Pred {
    Pred::TargetOs(s.into())
}

pub fn target_env(s: impl Into<String>) -> Pred {
    Pred::TargetEnv(s.into())
}

pub fn target_pointer_width(s: impl Into<String>) -> Pred {
    Pred::TargetPointerWidth(s.into())
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Any(x) => write!(f, "{x}"),
            Expr::All(x) => write!(f, "{x}"),
            Expr::Not(x) => write!(f, "{x}"),
            Expr::Atom(x) => write!(f, "{x}"),
        }
    }
}

impl fmt::Display for Any {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_list(f, "any", &self.0)
    }
}

impl fmt::Display for All {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_list(f, "all", &self.0)
    }
}

impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not({})", self.0)
    }
}

impl fmt::Display for Pred {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pred::TargetFamily(s) => match s.as_str() {
                "unix" | "windows" | "wasm" => write!(f, "{s}"),
                _ => fmt_pred(f, "target_family", s),
            },
            Pred::TargetVendor(s) => fmt_pred(f, "target_vendor", s),
            Pred::TargetArch(s) => fmt_pred(f, "target_arch", s),
            Pred::TargetOs(s) => fmt_pred(f, "target_os", s),
            Pred::TargetEnv(s) => fmt_pred(f, "target_env", s),
            Pred::TargetPointerWidth(s) => fmt_pred(f, "target_pointer_width", s),
        }
    }
}

fn fmt_pred(f: &mut fmt::Formatter<'_>, key: &str, value: &str) -> fmt::Result {
    write!(f, "{key} = {value:?}")
}

fn fmt_list(f: &mut fmt::Formatter<'_>, name: &str, list: &[Expr]) -> fmt::Result {
    let (x, xs) = list.split_first().expect("empty predicate list");
    write!(f, "{name}({x}")?;
    for x in xs {
        write!(f, ", {x}")?;
    }
    write!(f, ")")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cfg_string() {
        {
            let cfg = expr(target_family("unix"));
            let expected = "unix";
            assert_eq!(cfg.to_string(), expected);
        }
        {
            let cfg = expr(any((target_os("linux"), target_os("android"))));
            let expected = r#"any(target_os = "linux", target_os = "android")"#;
            assert_eq!(cfg.to_string(), expected);
        }
    }
}
