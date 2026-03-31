use std::{
    borrow::ToOwned,
    collections::{BTreeSet, HashSet},
    num::NonZeroU16,
    path::Path,
    sync::Arc,
};

use anyhow::Result;
use walkdir::WalkDir;

use crate::{room::ids::MapId, server::config::Config};

#[derive(Debug)]
pub struct Assets {
    config: Arc<Config>,

    pub maps: BTreeSet<MapId>,
    pub sprites: HashSet<Box<str>>,
    pub systems: HashSet<Box<str>>,
    pub sounds: HashSet<Box<str>>,
    pub pictures: HashSet<Box<str>>,
}

impl Assets {
    pub fn new(config: Arc<Config>) -> Result<Self> {
        let maps = Self::get_maps(config.as_ref())?;
        let sprites = Self::get_sprites(config.as_ref())?;
        let systems = Self::get_systems(config.as_ref())?;
        let sounds = Self::get_sounds(config.as_ref());
        let pictures = Self::get_pictures(config.as_ref())?;
        tracing::debug!(
            "loaded {} maps, {} sprites, {} systems, {} sounds, and {} pictures",
            maps.len(),
            sprites.len(),
            systems.len(),
            sounds.len(),
            pictures.len()
        );
        Ok(Self {
            config,
            maps,
            sprites,
            systems,
            sounds,
            pictures,
        })
    }
    fn get_maps(config: &Config) -> Result<BTreeSet<MapId>> {
        Ok(std::fs::read_dir(&config.game_path)?
            .filter_map(Result::ok)
            .map(|x| x.file_name())
            .filter_map(|x| x.to_str().map(ToOwned::to_owned))
            .filter(|x| x.len() == 11 && x[7..] == *".lmu")
            .filter_map(|x| x[3..7].parse().ok())
            .map(MapId)
            .collect())
    }
    fn get_sprites(config: &Config) -> Result<HashSet<Box<str>>> {
        get_stems("CharSet", &config.game_path)
    }
    fn get_systems(config: &Config) -> Result<HashSet<Box<str>>> {
        get_stems("System", &config.game_path)
    }
    fn get_sounds(config: &Config) -> HashSet<Box<str>> {
        let path = config.game_path.join("Sound");
        #[allow(clippy::unwrap_used)]
        WalkDir::new(&path)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|x| x.metadata().is_ok_and(|x| !x.is_dir()))
            .map(|x| x.path().strip_prefix(&path).unwrap().with_extension(""))
            .filter_map(|x| {
                Some([
                    x.to_str()?.into(),
                    x.to_str()?.replacen('/', "\\", 1).into_boxed_str(),
                ])
            })
            .flatten()
            .collect()
    }
    fn get_pictures(config: &Config) -> Result<HashSet<Box<str>>> {
        get_stems("Picture", &config.game_path)
    }
    pub fn is_valid_map_id(&self, id: NonZeroU16) -> Option<MapId> {
        let map_id = MapId(id);
        self.maps.contains(&map_id).then_some(map_id)
    }
    pub fn is_valid_sprite(&self, name: &str) -> bool {
        match name {
            "" => true,
            x if x.contains('/') || x.contains('\\') => false,
            x => self.sprites.contains(x),
        }
    }
    pub fn is_valid_system(&self, name: &str) -> bool {
        self.systems.contains(name)
    }
    pub fn is_valid_sound(&self, name: &str) -> bool {
        match name {
            name if name.contains("../") || name.contains("..\\") => false,
            name if self.config.bad_sounds.contains(name) => false,
            name => self.sounds.contains(name),
        }
    }
    pub fn is_valid_picture(&self, name: &str) -> bool {
        match name {
            name if name.contains('/') || name.contains('\\') => false,
            name if !self.pictures.contains(name) => false,
            name if self.config.pictures.contains(name) => true,
            name if self
                .config
                .picture_prefixes
                .iter()
                .any(|prefix| name.starts_with(&**prefix)) =>
            {
                true
            }
            _ => false,
        }
    }
}

fn get_stems(subdir: &str, game_path: &Path) -> Result<HashSet<Box<str>>> {
    let path = game_path.join(subdir);
    Ok(std::fs::read_dir(path)?
        .filter_map(|x| x.ok().map(|x| x.path()))
        .filter_map(|x| x.file_stem().map(ToOwned::to_owned))
        .filter_map(|x| Some(x.to_str()?.into()))
        .collect())
}
