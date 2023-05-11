use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cfg(pub Expr);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Expr {
    Any(Any),
    All(All),
    Not(Not),
    Atom(Pred),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct All(pub Vec<Expr>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Any(pub Vec<Expr>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Not(pub Box<Expr>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Pred {
    TargetFamily(String),
    TargetVendor(String),
    TargetArch(String),
    TargetOs(String),
    TargetEnv(String),
    TargetPointerWidth(String),
}

pub fn cfg(x: impl Into<Cfg>) -> Cfg {
    x.into()
}

pub fn expr(x: impl Into<Expr>) -> Expr {
    x.into()
}

pub fn any(x: impl Into<Any>) -> Any {
    x.into()
}

pub fn all(x: impl Into<All>) -> All {
    x.into()
}

#[macro_export]
macro_rules! any {
    ($($x:expr),+) => {{
        $crate::Any::from(vec![
            $($crate::Expr::from($x)),+
        ])
    }};
}

#[macro_export]
macro_rules! all {
    ($($x:expr),+) => {{
        $crate::All::from(vec![
            $($crate::Expr::from($x)),+
        ])
    }};
}

pub fn not<T>(expr: T) -> Not
where
    T: Into<Expr>,
{
    Not(Box::new(expr.into()))
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

impl From<Expr> for Cfg {
    fn from(expr: Expr) -> Self {
        Self(expr)
    }
}

impl From<Pred> for Cfg {
    fn from(pred: Pred) -> Self {
        Self(Expr::from(pred))
    }
}

impl From<Any> for Cfg {
    fn from(value: Any) -> Self {
        Self(Expr::from(value))
    }
}

impl From<All> for Cfg {
    fn from(value: All) -> Self {
        Self(Expr::from(value))
    }
}

impl From<Not> for Cfg {
    fn from(value: Not) -> Self {
        Self(Expr::from(value))
    }
}

impl From<Pred> for Expr {
    fn from(pred: Pred) -> Self {
        Expr::Atom(pred)
    }
}

impl From<Any> for Expr {
    fn from(value: Any) -> Self {
        Expr::Any(value)
    }
}

impl From<All> for Expr {
    fn from(value: All) -> Self {
        Expr::All(value)
    }
}

impl From<Not> for Expr {
    fn from(value: Not) -> Self {
        Expr::Not(value)
    }
}

impl From<Vec<Expr>> for Any {
    fn from(list: Vec<Expr>) -> Self {
        Self(list)
    }
}

impl From<Vec<Expr>> for All {
    fn from(list: Vec<Expr>) -> Self {
        Self(list)
    }
}

impl From<Box<Expr>> for Not {
    fn from(value: Box<Expr>) -> Self {
        Self(value)
    }
}
