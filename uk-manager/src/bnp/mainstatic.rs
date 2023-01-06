use anyhow::{Context, Result};
use fs_err as fs;
use roead::byml::{Byml, Hash};
use rustc_hash::FxHashMap;
use smartstring::alias::String;

use super::BnpConverter;

fn get_id(item: &Hash) -> Result<String> {
    #[inline]
    fn key_from_coords(x: f32, y: f32, z: f32) -> String {
        format!("{}{}{}", x.ceil(), y.ceil(), z.ceil()).into()
    }

    #[inline]
    fn find_name(item: &Hash) -> &str {
        item.iter()
            .find_map(|(k, v)| {
                k.to_lowercase()
                    .contains("name")
                    .then(|| v.as_string().ok().map(|v| v.as_str()))
                    .flatten()
            })
            .unwrap_or("")
    }

    let translate = item
        .get("Translate")
        .context("Mainfield static missing entry translation")?
        .as_hash()?;

    Ok(key_from_coords(
        translate
            .get("X")
            .context("Translate missing X")?
            .as_float()?,
        translate
            .get("Y")
            .context("Translate missing Y")?
            .as_float()?,
        translate
            .get("Z")
            .context("Translate missing Z")?
            .as_float()?,
    ) + find_name(item))
}

impl BnpConverter<'_> {
    pub fn handle_mainfield_static(&self) -> Result<()> {
        let mstatic_path = self.path.join("logs/mainstatic.yml");
        if mstatic_path.exists() {
            let diff: FxHashMap<String, Hash> = Byml::from_text(fs::read_to_string(mstatic_path)?)?
                .into_hash()?
                .into_iter()
                .map(|(cat, entries)| -> Result<(String, Hash)> { Ok((cat, entries.into_hash()?)) })
                .collect::<Result<_>>()?;
            let mut base: FxHashMap<String, Hash> = Byml::from_binary(
                self.dump()
                    .context("No dump for current mode")?
                    .get_aoc_bytes_uncached("Map/MainField/Static.smubin")?,
            )?
            .into_hash()?
            .into_iter()
            .map(|(cat, entries)| -> Result<(String, Hash)> {
                let entries = entries
                    .into_array()?
                    .into_iter()
                    .map(|entry| -> Result<(String, Byml)> {
                        Ok((get_id(entry.as_hash()?)?, entry))
                    })
                    .collect::<Result<_>>()?;
                Ok((cat, entries))
            })
            .collect::<Result<_>>()?;
            for (cat, entries) in diff {
                base.get_mut(&cat)
                    .context("Base mainfield static missing category")?
                    .extend(entries.into_iter());
            }
            let output: Byml = base
                .into_iter()
                .map(|(cat, entries)| (cat, entries.into_values().collect()))
                .collect();
            let dest_path = self.path.join(self.aoc).join("Map/MainField/Static.smubin");
            fs::write(dest_path, output.to_binary(self.platform.into()))?;
        }
        Ok(())
    }
}
