use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use serde_json::{Value, Map};
use color_eyre::Result;
use color_eyre::eyre::bail;
use std::fs;
use std::path::Path;
use std::fs::File;
use std::io::Read;
use log::debug;
use std::env;
use jiff::{Unit, Zoned};

fn git_status(repo_root: &Path) -> Result<String> {
    let repo = gix::open(repo_root)?;
    let status = match repo.head_commit() {
        Ok(commit) => {
            let id = commit.short_id().unwrap();;

            let r#ref = match repo.head_ref().unwrap() {
                Some(r#ref) => {
                    r#ref.name().shorten().to_string()
                }
                None => {String::from("detached")}
            };

            let mut status = format!("{}({})", id, r#ref);

            if repo.is_dirty().unwrap() {
                status.push_str("-dirty");
            }       

            status
        },
        Err(_) => {
            String::from("NoCommitFound")
        }
    };
    Ok(status)
}

fn name() -> Result<String> {    
    let name = petname::petname(2, "-").unwrap();
    Ok(name)
}

fn find_repo_root() -> Result<PathBuf> {
    let cwd = env::current_dir()?;
    let mut ancestors = cwd.ancestors();

    while let Some(cwd) = ancestors.next() {
        for entry in fs::read_dir(cwd)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                let name = name.to_str().unwrap();
                if name == ".git" {
                    return Ok(cwd.to_path_buf());
                }
            }
        }
    }
    bail!("No .git dir found in cwd ancestory");
}

fn timestamp() -> Result<String> {
    let timestamp = Zoned::now()
        .strftime("%Y-%m-%d-%H:%M:%S(%Z)")
        .to_string();
    Ok(timestamp)
}

#[derive(Debug)]
pub enum RepoRootConfig {
    Discover,
    Path(PathBuf)
}
impl RepoRootConfig {
    pub fn from_option(x: Option<PathBuf>) -> Self {
        match x {
            Some(p) => RepoRootConfig::Path(p),
            None => RepoRootConfig::Discover,
        }
    }
}

pub fn label(repo_path: RepoRootConfig) -> Result<String> {
    let name = name()?;
    let repo_root = match repo_path {
        RepoRootConfig::Discover => find_repo_root()?,
        RepoRootConfig::Path(p) => p,
    };
    let git_status = git_status(&repo_root)?; 
    let timestamp = timestamp()?;
    let label = format!("{}-{}-{}", name, timestamp, git_status);
	Ok(label)
}
