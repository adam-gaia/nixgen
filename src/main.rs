use color_eyre::Result;
use color_eyre::eyre::bail;
use log::debug;
use log::info;
use log::error;
use std::fs;
use std::path::PathBuf;
use std::path::Path;
use directories::BaseDirs;
use std::process::Command;
use jiff::{Zoned, Unit};
use std::env;
use clap::{Parser, Args, Subcommand, ValueEnum};
use nixgen::NixGen;
use nixgen::label;
use nixgen::RepoRootConfig;

#[derive(Debug, Subcommand)]
enum Commands {
	/// List all NixOS generations
	List,

	/// Show the current NixOS generation
	Current,

	/// Generate a label for a generation
	Label {
		repo_root: Option<PathBuf>
	},
}

#[derive(Debug, Parser)]
#[clap(version)]
struct Cli {
	/// Action to perform
	#[command(subcommand)]
	command: Option<Commands>,
}

fn list_all(nixgen: &NixGen) -> Result<()> {
	let mut generations = nixgen.all_generations()?;
	generations.sort_by_key(|x| x.0);
	for (index, generation) in generations {
		let label = generation.label();
		println!("{}: {}", index, label);
	}
	Ok(())
}

fn current(nixgen: &NixGen) -> Result<()> {
	let generation = nixgen.current_generation()?;
	let label = generation.label();
	println!("{}", label);
	Ok(())
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Cli::parse();

    let nixgen = NixGen::default()?;
	match args.command {
		Some(command) => {
			match command {
				Commands::List => {
					list_all(&nixgen)?;
				},
				Commands::Current => {
					current(&nixgen)?;
				},
				Commands::Label {repo_root}=> {
					let config = RepoRootConfig::from_option(repo_root);
				    let label = label(config)?;	
				    println!("{}", label);
				},
			}
		},
		None => {
			current(&nixgen)?;
		}
	}

	Ok(())
}
