use anyhow::{anyhow, Result};
use fs_err as fs;
use once_cell::sync::Lazy;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use uk_reader::ResourceReader;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Platform {
    WiiU,
    Switch,
}

impl From<roead::Endian> for Platform {
    fn from(e: roead::Endian) -> Self {
        match e {
            roead::Endian::Big => Self::WiiU,
            roead::Endian::Little => Self::Switch,
        }
    }
}

impl From<Platform> for roead::Endian {
    fn from(p: Platform) -> Self {
        match p {
            Platform::WiiU => Self::Big,
            Platform::Switch => Self::Little,
        }
    }
}

impl From<uk_content::prelude::Endian> for Platform {
    fn from(e: uk_content::prelude::Endian) -> Self {
        match e {
            uk_content::prelude::Endian::Big => Self::WiiU,
            uk_content::prelude::Endian::Little => Self::Switch,
        }
    }
}

impl From<Platform> for uk_content::prelude::Endian {
    fn from(p: Platform) -> Self {
        match p {
            Platform::WiiU => Self::Big,
            Platform::Switch => Self::Little,
        }
    }
}

impl From<rstb::Endian> for Platform {
    fn from(e: rstb::Endian) -> Self {
        match e {
            rstb::Endian::Big => Self::WiiU,
            rstb::Endian::Little => Self::Switch,
        }
    }
}

impl From<Platform> for rstb::Endian {
    fn from(p: Platform) -> Self {
        match p {
            Platform::WiiU => Self::Big,
            Platform::Switch => Self::Little,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Language {
    USen,
    EUen,
    USfr,
    USes,
    EUde,
    EUes,
    EUfr,
    EUit,
    EUnl,
    EUru,
    CNzh,
    JPja,
    KRko,
    TWzh,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployConfig {
    pub output: PathBuf,
    pub method: DeployMethod,
    pub auto: bool,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeployMethod {
    Copy,
    HardLink,
    Symlink,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformSettings {
    pub dump: Arc<ResourceReader>,
    pub deploy_config: Option<DeployConfig>,
    pub language: Language,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub current_mode: Platform,
    pub storage_dir: PathBuf,
    pub unpack_mods: bool,
    pub wiiu_config: Option<PlatformSettings>,
    pub switch_config: Option<PlatformSettings>,
    pub check_updates: bool,
    pub show_changelog: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            current_mode: Platform::WiiU,
            storage_dir: dirs2::config_dir().unwrap().join("ukmm"),
            unpack_mods: false,
            wiiu_config: None,
            switch_config: None,
            check_updates: true,
            show_changelog: true,
        }
    }
}

impl Settings {
    pub fn load() -> Arc<RwLock<Settings>> {
        Arc::new(RwLock::new(match Settings::read(&SETTINGS_PATH) {
            Ok(settings) => {
                log::debug!("{:?}", settings);
                settings
            }
            Err(e) => {
                log::error!("Failed to read settings file:\n{}", e);
                log::info!("Loading default settings instead");
                Settings::default()
            }
        }))
    }

    pub fn read(path: &Path) -> Result<Self> {
        Ok(toml::from_str(&fs::read_to_string(path)?)?)
    }

    pub fn save(&self) -> Result<()> {
        if !SETTINGS_PATH.parent().unwrap().exists() {
            fs::create_dir_all(SETTINGS_PATH.parent().unwrap())?;
        }
        log::debug!("Saving settings:\n{:?}", self);
        fs::write(SETTINGS_PATH.as_path(), toml::to_string_pretty(self)?)?;
        log::info!("Settings saved");
        Ok(())
    }

    #[inline]
    pub fn platform_dir(&self) -> PathBuf {
        self.get_platform_dir(self.current_mode)
    }

    #[inline]
    pub fn get_platform_dir(&self, platform: Platform) -> PathBuf {
        match platform {
            Platform::Switch => self.storage_dir.join("nx"),
            Platform::WiiU => self.storage_dir.join("wiiu"),
        }
    }

    #[inline]
    pub fn mods_dir(&self) -> PathBuf {
        self.platform_dir().join("mods")
    }

    #[inline]
    pub fn dump(&self) -> Option<Arc<ResourceReader>> {
        match self.current_mode {
            Platform::Switch => self.switch_config.as_ref().map(|c| c.dump.clone()),
            Platform::WiiU => self.wiiu_config.as_ref().map(|c| c.dump.clone()),
        }
    }

    #[inline(always)]
    pub fn platform_config(&self) -> Option<&PlatformSettings> {
        match self.current_mode {
            Platform::Switch => self.switch_config.as_ref(),
            Platform::WiiU => self.wiiu_config.as_ref(),
        }
    }

    #[inline]
    pub fn merged_dir(&self) -> PathBuf {
        self.platform_dir().join("merged")
    }

    #[inline]
    pub fn deploy_dir(&self) -> Option<&Path> {
        let config = self.platform_config();
        config
            .and_then(|c| c.deploy_config.as_ref())
            .map(|c| c.output.as_ref())
    }
}

static SETTINGS_PATH: Lazy<PathBuf> = Lazy::new(|| {
    if std::env::args().any(|a| a == "--portable") {
        std::env::current_dir().unwrap().join("settings.toml")
    } else {
        dirs2::config_dir().unwrap().join("ukmm/settings.toml")
    }
});
