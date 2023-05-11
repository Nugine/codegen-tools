use libc_cfg::generate_item_cfg;
use libc_cfg::generate_item_list;

use anyhow::Result;
use camino::Utf8PathBuf;
use clap::Parser;
use regex::RegexSet;

#[derive(clap::Parser)]
struct Opt {
    #[clap(long)]
    libc: Utf8PathBuf,

    filters: Vec<String>,
}

fn main() -> Result<()> {
    env_logger::init();
    let opt = Opt::parse();

    let libc_repo_path = &opt.libc;

    let re = RegexSet::new(&opt.filters)?;

    let item_list = generate_item_list(libc_repo_path)?;

    for item in &item_list {
        let name = item.name.as_str();
        if re.is_match(name) {
            let cfg = generate_item_cfg(item);
            println!("#[{cfg}]\npub use libc::{name};\n");
        }
    }

    Ok(())
}
