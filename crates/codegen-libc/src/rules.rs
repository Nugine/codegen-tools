use codegen_cfg::ast::*;

use std::collections::HashMap;

use once_cell::sync::Lazy;
use rust_utils::default::default_with;
use rust_utils::iter::map_collect_vec;

pub static PROMOTE_MODS: &[&str] = &[
    "align",          // rustc >= 1.25
    "int128",         // rustc >= 1.26
    "non_exhaustive", // rustc >= 1.40
    "long_array",     // rustc >= 1.47
    "freebsd10",
    "freebsd11",
    "freebsd12",
    "freebsd13",
    "freebsd14",
    "native",   // unix/haiku/native
    "arch",     // unix/linux_like/linux/arch
    "generic",  // unix/linux_like/linux/arch/generic
    "other",    // unix/linux_like/linux/uclibc/x86_64/other
    "neutrino", // unix/nto/neutrino
];

type Rules = HashMap<String, Expr>;

pub static MATCH_ALL: Lazy<Rules> = Lazy::new(|| {
    default_with::<Rules>(|ans| {
        let mut add = |mod_path: &str, expr: Expr| assert!(ans.insert(mod_path.to_owned(), expr).is_none());

        for s in ["unix", "windows", "wasm"] {
            add(s, expr(target_family(s)));
        }

        add("sgx", expr(all((target_env("sgx"), target_vendor("fortanix")))));

        add("wasi", expr(any((target_env("wasi"), target_os("wasi")))));
    })
});

static OS_LIST: &[&str] = &[
    "linux",
    "windows",
    "fuchsia",
    "switch",
    "psp",
    "vxworks",
    "hermit",
    "xous",
    "aix",
    "dragonfly",
    "freebsd",
    "openbsd",
    "netbsd",
    "haiku",
    "android",
    "emscripten",
    "l4re",
    "espidf",
    "horizon",
    "vita",
    "nto",
    "redox",
    "illumos",
    "solaris",
];

static ARCH_LIST: &[&str] = &[
    "x86",
    "x86_64",
    "arm",
    "aarch64",
    "mips",
    "mips64",
    "powerpc",
    "powerpc64",
    "riscv32",
    "riscv64",
    "loongarch64",
    "sparc",
    "sparc64",
    "m68k",
    "s390x",
    "hexagon",
];

static ENV_LIST: &[&str] = &["gnu", "msvc", "musl", "newlib", "uclibc"];

pub static MATCH_COMPONENT: Lazy<Rules> = Lazy::new(|| {
    default_with::<Rules>(|ans| {
        let mut add = |comp: &str, expr: Expr| assert!(ans.insert(comp.to_owned(), expr).is_none());

        {
            // family, exact match
            let s = "unix";
            add(s, expr(target_family(s)));
        }

        {
            // os, exact match
            for &s in OS_LIST {
                add(s, expr(target_os(s)));
            }
        }

        {
            // arch, exact match
            for &s in ARCH_LIST {
                add(s, expr(target_arch(s)));
            }

            add("mips32", expr(target_arch("mips")));
            add("x86_common", expr(any((target_arch("x86"), target_arch("x86_64")))));
        }

        {
            // env, exact match
            for &s in ENV_LIST {
                add(s, expr(target_env(s)));
            }
        }

        {
            // pointer width
            add("b32", expr(target_pointer_width("32")));
            add("b64", expr(target_pointer_width("64")));

            add("ilp32", expr(target_pointer_width("32")));
            add("lp64", expr(target_pointer_width("64")));

            add("x32", expr(target_pointer_width("32")));
            add("not_x32", expr(target_pointer_width("64")));
        }

        {
            // groups

            add(
                "bsd",
                any_target_os(&[
                    "macos",
                    "ios",
                    "tvos",
                    "watchos",
                    "freebsd",
                    "dragonfly",
                    "openbsd",
                    "netbsd",
                ]),
            );

            add("apple", any_target_os(&["macos", "ios", "tvos", "watchos"]));

            add("netbsdlike", any_target_os(&["openbsd", "netbsd"]));

            add("freebsdlike", any_target_os(&["freebsd", "dragonfly"]));

            add("linux_like", any_target_os(&["linux", "l4re", "android", "emscripten"]));

            add("solarish", any_target_os(&["solaris", "illumos"]));
        }

        {
            // special cases
            add("solid", expr(target_os("solid_asp3")));
        }
    })
});

fn any_target_os(os_list: &[&str]) -> Expr {
    expr(any(map_collect_vec(os_list, |&s| expr(target_os(s)))))
}
