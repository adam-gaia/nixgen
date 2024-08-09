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
use once_cell::sync::Lazy;
use regex::Regex;

const DEFAULT_PROFILE_DIR: &str = "/nix/var/nix/profiles";
const DEFAULT_SYSTEM_PROFILE: &str = "system";

/// NixOS bootspec
/// https://github.com/NixOS/rfcs/blob/master/rfcs/0125-bootspec.md
#[derive(Serialize, Deserialize)]
struct Bootspec {
    init: PathBuf,
    initrd: Option<PathBuf>,
    initSecrets: Option<PathBuf>,
    kernel: PathBuf,
    kernelParams: Vec<String>, // TODO: maybe create custom type with custom ser/de to parse kernel command line options
    label: String,
    system: String,
    #[serde(rename = "topLevel")]
    top_level: Option<PathBuf>, // RFC says this is required, but its not there in my current system generation
}

#[derive(Serialize, Deserialize)]
pub struct Generation {
    #[serde(rename = "org.nixos.bootspec.v1")]
    bootspec: Bootspec,

    #[serde(rename = "org.nixos.specialisation.v1")]
    specialisation: Map<String, Value>,

    // TODO: how could we handle other top level keysi? From the RFC: "The top-level object may contain arbitrary further keys ("extensions"), whose semantics may be defined by third parties."
}

impl Generation {
    pub fn label(&self) -> String {
        self.bootspec.label.clone()
    }
}

pub struct NixGen {
    profiles_dir: PathBuf,
}

impl NixGen {
    pub fn default() -> Result<Self> {
        Ok(Self {
           profiles_dir: PathBuf::from(DEFAULT_PROFILE_DIR),
        })
    }

    pub fn new(profiles_dir: &Path) -> Result<Self> {
        Ok(
            Self {
                profiles_dir: profiles_dir.to_path_buf()
            }
        )
    }

    fn generation_from_path(&self, path: &Path) -> Result<Generation> {    
        let mut path = path.to_path_buf();
        if path.is_dir() {
            path = path.join("boot.json");
        }
        debug!("{}", &path.display());
        let path = fs::canonicalize(&path)?;
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let generation: Generation = serde_json::from_str(&contents)?;
        Ok(generation)
    }

    pub fn current_generation(&self) -> Result<Generation> {
        let profile_dir_symlink = self.profiles_dir.join(DEFAULT_SYSTEM_PROFILE);
        let link_target = match fs::canonicalize(&profile_dir_symlink) {
            Ok(link_target) => {link_target},
            Err(e) => {
                bail!("Unable to resolve system profile symlink '{}': {}", profile_dir_symlink.display(), e)
            },
        };
        let generation = self.generation_from_path(&link_target)?;
        Ok(generation)
    }

    

    pub fn all_generations(&self) -> Result<Vec<(usize, Generation)>> {
        let entries = fs::read_dir(&self.profiles_dir)?;
        let mut generations = Vec::new();
        for entry in entries {
            let entry = entry?;
            let name = entry.file_name();
            static re: Lazy<Regex> = Lazy::new(|| Regex::new(r"system-([0-9]+)-link").unwrap());
            let Some(capture_groups) = re.captures(name.to_str().unwrap()) else {
                continue;
            };
            let Some(index_match) = capture_groups.get(1) else {
                bail!("Bad regex match");
            };
            let index: usize = index_match.as_str().parse().unwrap();
            let path = entry.path();
            let generation = self.generation_from_path(&path)?;
            generations.push((index, generation));
        }
        Ok(generations)
    }
}
 
