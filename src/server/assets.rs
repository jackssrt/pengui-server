use anyhow::Result;
use std::{collections::HashSet, path::Path, sync::Arc};
use walkdir::WalkDir;

use crate::{room::ids::MapId, server::config::Config};

pub struct Assets {
    config: Arc<Config>,
    maps: Vec<MapId>,

    sprites: HashSet<String>,
    systems: HashSet<String>,
    sounds: HashSet<String>,
    pictures: HashSet<String>,
}

impl Assets {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let maps = Self::get_maps(config.as_ref())?;
        let sprites = Self::get_sprites(config.as_ref())?;
        let systems = Self::get_systems(config.as_ref())?;
        let sounds = Self::get_sounds(config.as_ref());
        let pictures = Self::get_pictures(config.as_ref())?;
        Ok(Self {
            config,
            maps,
            sprites,
            systems,
            sounds,
            pictures,
        })
    }
    fn get_maps(config: &Config) -> Result<Vec<MapId>> {
        Ok(std::fs::read_dir(&config.game_path)?
            .filter_map(Result::ok)
            .map(|x| x.file_name())
            .filter_map(|x| Some(x.to_str()?.to_owned()))
            .filter(|x| x.len() == 11 && x[7..] == *".lmu")
            .filter_map(|x| x.parse().ok())
            .map(MapId)
            .collect())
    }
    fn get_sprites(config: &Config) -> Result<HashSet<String>> {
        get_stems("CharSet", &config.game_path)
    }
    fn get_systems(config: &Config) -> Result<HashSet<String>> {
        get_stems("System", &config.game_path)
    }
    fn get_sounds(config: &Config) -> HashSet<String> {
        let path = config.game_path.join("Sounds");
        WalkDir::new(path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|x| !x.metadata().is_ok_and(|x| x.is_dir()))
            .map(|x| x.path().with_extension(""))
            .filter_map(|x| {
                Some([
                    x.to_string_lossy().into(),
                    x.to_str()?.replacen('/', "\\", 1),
                ])
            })
            .flatten()
            .collect()
    }
    fn get_pictures(config: &Config) -> Result<HashSet<String>> {
        get_stems("Picture", &config.game_path)
    }
    fn is_valid_sprite(&self, name: &str) -> bool {
        match name {
            "" => true,
            x if x.contains('/') || x.contains('\\') => false,
            x => self.sprites.contains(x),
        }
    }
    fn is_valid_system(&self, name: &String, ignore_single_quotes: bool) -> bool {
        let name = {
            if ignore_single_quotes {
                &name.replace('\'', "")
            } else {
                name
            }
        };
        self.systems.contains(name)
    }
    fn is_valid_sound(&self, name: &str) -> bool {
        match name {
            name if name.contains("../") || name.contains("..\\") => false,
            name if self.config.bad_sounds.contains(name) => false,
            name => self.sounds.contains(name),
        }
    }
    fn is_valid_picture(&self, name: &String) -> bool {
        match name {
            name if name.contains('/') || name.contains('\\') => false,
            name if !self.pictures.contains(name) => false,
            name if self.config.pictures.contains(name) => true,
            name if self
                .config
                .picture_prefixes
                .iter()
                .any(|prefix| name.starts_with(prefix)) =>
            {
                true
            }
            _ => false,
        }
    }
}

fn get_stems(subdir: &str, game_path: &Path) -> Result<HashSet<String>> {
    let path = game_path.join(subdir);
    Ok(std::fs::read_dir(path)?
        .filter_map(|x| x.ok().map(|x| x.path()))
        .filter_map(|x| x.file_stem().map(std::borrow::ToOwned::to_owned))
        .filter_map(|x| Some(x.to_str()?.to_owned()))
        .collect())
}
