use clap::CommandFactory;
use clap_complete::{
    generate_to,
    shells::{Bash, Fish, Zsh},
};
use clap_mangen::Man;
use std::fs;

include!("src/args.rs");

fn main() -> std::io::Result<()> {
    let out_dir = "target/assets";
    fs::create_dir_all(out_dir)?;

    let cmd = Args::command();
    let name = "gremlh";

    let man = Man::new(cmd.clone());
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    fs::write(Path::new(out_dir).join(format!("{}.1", name)), buffer)?;

    generate_to(Bash, &mut cmd.clone(), name, out_dir)?;
    generate_to(Zsh, &mut cmd.clone(), name, out_dir)?;
    generate_to(Fish, &mut cmd.clone(), name, out_dir)?;

    println!("cargo:warning=Assets generated in {}", out_dir);
    println!("cargo:rerun-if-changed=src/args.rs");

    Ok(())
}
