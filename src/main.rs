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

#[derive(Debug, Subcommand)]
enum Commands {
	/// List all NixOS generations
	List,
	/// Show the current NixOS generation
	Current,
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
				}		
			}
		},
		None => {
			current(&nixgen)?;
		}
	}

	Ok(())
}
