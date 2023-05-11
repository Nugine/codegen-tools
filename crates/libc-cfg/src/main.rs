mod utils;
use self::utils::*;

mod rules;

use target_cfg::*;

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::ops::Not;

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use clap::Parser;
use log::error;
use regex::{Regex, RegexSet};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

#[derive(Debug)]
struct SourceFile {
    mod_path: Utf8PathBuf,
    fs_path: Utf8PathBuf,
}

fn load_src_list(libc_repo_path: impl AsRef<Utf8Path>) -> Result<Vec<SourceFile>> {
    let mut ans = Vec::new();

    let src_dir = libc_repo_path.as_ref().join("src");
    let walk = WalkDir::new(&src_dir);
    for entry in walk {
        let entry = entry?;
        let fs_path: Utf8PathBuf = entry.path().to_owned().try_into()?;

        if fs_path.extension() != Some("rs") {
            continue;
        }

        let rel_path = fs_path.strip_prefix(&src_dir)?.to_owned();

        let mod_path = if rel_path.file_name() == Some("mod.rs") {
            rel_path.parent().unwrap().to_owned()
        } else {
            rel_path.with_extension("")
        };

        let ignored_tail = [
            "fixed_width_ints", // TODO: int128
            "no_align",         // cfg(libc_align), unused when rustc >= 1.25
            "errno",            // cfg(libc_thread_local), unstable
        ];
        if ignored_tail.iter().copied().any(|s| mod_path.ends_with(s)) {
            continue;
        }

        let mod_path = {
            let promote = rules::PROMOTE_MODS;

            let mut components = mod_path.components().collect::<Vec<_>>();
            components.retain(|c| promote.contains(&c.as_str()).not());
            components.into_iter().collect::<Utf8PathBuf>()
        };

        ans.push(SourceFile { fs_path, mod_path });
    }

    ans.sort_by(|lhs, rhs| lhs.mod_path.cmp(&rhs.mod_path));

    Ok(ans)
}

fn load_item_names(src: &SourceFile) -> Result<Vec<String>> {
    let mut ans = Vec::new();

    let content = fs::read_to_string(&src.fs_path)?;

    let re = Regex::new(r"^\s*pub (type|const|struct|union|fn) ([A-Za-z0-9_]+)").unwrap();

    for line in content.lines() {
        if let Some(caps) = re.captures(line) {
            let item = caps.get(2).unwrap().as_str();
            ans.push(item.to_owned());
        }
    }

    ans.sort_unstable();

    Ok(ans)
}

#[derive(Debug, Serialize, Deserialize)]
struct Item {
    name: String,
    mod_paths: BTreeSet<Utf8PathBuf>,
    any: Vec<Expr>,
}

fn generate_item_list(libc_repo_path: impl AsRef<Utf8Path>) -> Result<Vec<Item>> {
    let mut map: HashMap<String, Item> = default();

    let src_list = load_src_list(libc_repo_path)?;

    for src in src_list {
        let item_list = load_item_names(&src)?;

        for name in item_list {
            let item = map.entry(name.clone()).or_insert_with(|| Item {
                name: name.clone(),
                mod_paths: default(),
                any: default(),
            });

            let is_new_mod_path = item.mod_paths.insert(src.mod_path.clone());

            if is_new_mod_path {
                let expr = generate_cfg_expr(&src.mod_path, &name);
                item.any.push(expr);
            }
        }
    }

    let mut list: Vec<_> = map.into_values().collect();
    list.sort_unstable_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    Ok(list)
}

fn generate_cfg_expr(mod_path: &Utf8Path, item_name: &str) -> Expr {
    let match_all_rules = &*rules::MATCH_ALL;
    if let Some(expr) = match_all_rules.get(mod_path.as_str()) {
        return expr.clone();
    }

    let components: Vec<_> = mod_path.components().collect();
    let mut components: Vec<&str> = components.iter().map(|c| c.as_str()).collect();
    assert!(components.is_empty().not());

    if components.len() > 1 && components.first().copied() == Some("unix") {
        components.remove(0);
    }

    let match_component_rules = &*rules::MATCH_COMPONENT;
    let mut conds: Vec<Expr> = Vec::new();

    for &s in &components {
        if let Some(expr) = match_component_rules.get(s) {
            conds.push(expr.clone());
            continue;
        }

        // error ------------------------------

        error!("item_name: {item_name}");
        error!("component: {s}");
        error!("mod_path:  {mod_path}");

        unimplemented!("unknown component")
    }

    if conds.is_empty() {
        error!("item_name: {item_name}");
        error!("mod_path:  {mod_path}");
        unimplemented!("empty conditions")
    }

    let mut ans = expr(all(conds));
    target_cfg::simplify(&mut ans);
    ans
}

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    libc: Utf8PathBuf,

    #[clap(long)]
    cache: Utf8PathBuf,

    #[clap(long)]
    force: bool,

    filters: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::parse();

    let libc_repo_path = &opt.libc;
    let cache_path = &opt.cache;

    anyhow::ensure!(matches!(cache_path.extension(), Some("json") | Some("bin")));

    let re = RegexSet::new(&opt.filters)?;

    let item_list = if opt.force || cache_path.exists().not() {
        let item_list = generate_item_list(libc_repo_path)?;

        let ext = cache_path.extension().unwrap();
        if ext == "json" {
            write_json(cache_path, &item_list)?;
        } else {
            write_bincode(cache_path, &item_list)?;
        }

        item_list
    } else {
        let ext = cache_path.extension().unwrap();

        if ext == "json" {
            read_json(cache_path)?
        } else {
            read_bincode(cache_path)?
        }
    };

    for item in &item_list {
        let name = item.name.as_str();
        if re.is_match(name) {
            let cfg = cfg(any(item.any.clone()));
            println!("#[{cfg}]\npub use libc::{name};\n");
        }
    }

    Ok(())
}
