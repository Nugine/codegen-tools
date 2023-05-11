use super::*;

use std::fmt;

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
            let cfg = expr(any!(target_os("linux"), target_os("android")));
            let expected = r#"any(target_os = "linux", target_os = "android")"#;
            assert_eq!(cfg.to_string(), expected);
        }
    }
}
