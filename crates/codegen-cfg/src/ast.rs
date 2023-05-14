use std::fmt;

pub use bool_logic::ast::All;
pub use bool_logic::ast::Any;
pub use bool_logic::ast::Not;
pub use bool_logic::ast::Var;

pub type Expr = bool_logic::ast::Expr<Pred>;

pub fn expr(x: impl Into<Expr>) -> Expr {
    x.into()
}

pub fn any(x: impl Into<Any<Pred>>) -> Any<Pred> {
    x.into()
}

pub fn all(x: impl Into<All<Pred>>) -> All<Pred> {
    x.into()
}

pub fn not(x: impl Into<Not<Pred>>) -> Not<Pred> {
    x.into()
}

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

impl From<Pred> for Expr {
    fn from(x: Pred) -> Self {
        Expr::Var(Var(x))
    }
}

impl fmt::Display for Pred {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pred::TargetFamily(s) => match s.as_str() {
                "unix" | "windows" | "wasm" => write!(f, "{}", s),
                _ => fmt_kv(f, "target_family", s),
            },
            Pred::TargetVendor(s) => fmt_kv(f, "target_vendor", s),
            Pred::TargetArch(s) => fmt_kv(f, "target_arch", s),
            Pred::TargetOs(s) => fmt_kv(f, "target_os", s),
            Pred::TargetEnv(s) => fmt_kv(f, "target_env", s),
            Pred::TargetPointerWidth(s) => fmt_kv(f, "target_pointer_width", s),
        }
    }
}

fn fmt_kv(f: &mut fmt::Formatter<'_>, key: &str, value: &str) -> fmt::Result {
    write!(f, "{key} = {value:?}")
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
