mod utils;
use self::utils::*;

mod rules;

use target_cfg::*;

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::ops::Not;

use anyhow::Result;
use camino::{Utf8Path, Utf8PathBuf};
use log::error;
use regex::Regex;
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
pub struct Item {
    pub name: String,
    pub mod_paths: BTreeSet<Utf8PathBuf>,
}

pub fn generate_item_list(libc_repo_path: impl AsRef<Utf8Path>) -> Result<Vec<Item>> {
    let mut map: HashMap<String, Item> = default();
    let src_list = load_src_list(libc_repo_path)?;

    for src in src_list {
        let item_list = load_item_names(&src)?;

        for name in item_list {
            let item = map.entry(name.clone()).or_insert_with(|| Item {
                name,
                mod_paths: default(),
            });

            item.mod_paths.insert(src.mod_path.clone());
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

    expr(all(conds))
}

pub fn generate_item_cfg(item: &Item) -> Expr {
    let conds = map_collect_vec(&item.mod_paths, |mod_path| generate_cfg_expr(mod_path, &item.name));
    simplified_expr(any(conds))
}
