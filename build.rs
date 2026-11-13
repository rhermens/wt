use std::{env, io, path::PathBuf};

use clap::CommandFactory;

#[path = "src/cli.rs"]
mod cli;

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src/cli.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").ok_or(io::ErrorKind::NotFound)?);
    clap_mangen::generate_to(cli::Args::command(), out_dir)?;

    Ok(())
}
