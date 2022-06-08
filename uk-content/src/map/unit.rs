use crate::{prelude::Mergeable, util::SortedDeleteMap, Result, UKError};
use roead::byml::Byml;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub struct MapUnit {
    pub pos_x: Option<f32>,
    pub pos_y: Option<f32>,
    pub size: Option<f32>,
    pub objects: SortedDeleteMap<u32, Byml>,
    pub rails: SortedDeleteMap<u32, Byml>,
}

impl TryFrom<&Byml> for MapUnit {
    type Error = UKError;

    fn try_from(byml: &Byml) -> Result<Self> {
        let hash = byml.as_hash()?;
        Ok(Self {
            pos_x: hash
                .get("LocationPosX")
                .map(|v| -> Result<f32> { Ok(v.as_float()?) })
                .transpose()?,
            pos_y: hash
                .get("LocationPosY")
                .map(|v| -> Result<f32> { Ok(v.as_float()?) })
                .transpose()?,
            size: hash
                .get("LocationSize")
                .map(|v| -> Result<f32> { Ok(v.as_float()?) })
                .transpose()?,
            objects: hash
                .get("Objs")
                .ok_or(UKError::MissingBymlKey("Map unit missing objs"))?
                .as_array()?
                .iter()
                .map(|obj| -> Result<(u32, Byml)> {
                    let hash = obj.as_hash()?;
                    let id = hash
                        .get("HashId")
                        .ok_or(UKError::MissingBymlKey("Map unit object missing hash ID"))?
                        .as_uint()?;
                    Ok((id, obj.clone()))
                })
                .collect::<Result<_>>()?,
            rails: hash
                .get("Rails")
                .ok_or(UKError::MissingBymlKey("Map unit missing rails"))?
                .as_array()?
                .iter()
                .map(|obj| -> Result<(u32, Byml)> {
                    let hash = obj.as_hash()?;
                    let id = hash
                        .get("HashId")
                        .ok_or(UKError::MissingBymlKey("Map unit rail missing hash ID"))?
                        .as_uint()?;
                    Ok((id, obj.clone()))
                })
                .collect::<Result<_>>()?,
        })
    }
}

impl From<MapUnit> for Byml {
    fn from(val: MapUnit) -> Self {
        [
            (
                "Objs",
                val.objects.into_iter().map(|(_, obj)| obj).collect(),
            ),
            ("Rails", val.rails.into_iter().map(|(_, obj)| obj).collect()),
        ]
        .into_iter()
        .chain(
            [
                ("LocationPosX", val.pos_x),
                ("LocationPosY", val.pos_y),
                ("LocationSize", val.size),
            ]
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, Byml::Float(v)))),
        )
        .collect()
    }
}

impl Mergeable<Byml> for MapUnit {
    fn diff(&self, other: &Self) -> Self {
        Self {
            pos_x: other.pos_x,
            pos_y: other.pos_y,
            size: other.size,
            objects: self.objects.diff(&other.objects),
            rails: self.rails.diff(&other.rails),
        }
    }

    fn merge(&self, diff: &Self) -> Self {
        Self {
            pos_x: diff.pos_x,
            pos_y: diff.pos_y,
            size: diff.size,
            objects: self.objects.merge(&diff.objects),
            rails: self.rails.merge(&diff.rails),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use roead::byml::Byml;

    fn load_cdungeon_munt() -> Byml {
        Byml::from_binary(
            &roead::yaz0::decompress(
                &std::fs::read("test/Map/CDungeon/Dungeon044/Dungeon044_Static.smubin").unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn load_mod_cdungeon_munt() -> Byml {
        Byml::from_binary(
            &roead::yaz0::decompress(
                &std::fs::read("test/Map/CDungeon/Dungeon044/Dungeon044_Static.mod.smubin")
                    .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn load_mainfield_munt() -> Byml {
        Byml::from_binary(
            &roead::yaz0::decompress(
                &std::fs::read("test/Map/MainField/D-3/D-3_Dynamic.smubin").unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn load_mod_mainfield_munt() -> Byml {
        Byml::from_binary(
            &roead::yaz0::decompress(
                &std::fs::read("test/Map/MainField/D-3/D-3_Dynamic.mod.smubin").unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
    }

    #[test]
    fn serde_mainfield() {
        let byml = load_mainfield_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let data = Byml::from(munt.clone()).to_binary(roead::Endian::Big);
        let byml2 = Byml::from_binary(&data).unwrap();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        assert_eq!(munt, munt2);
    }

    #[test]
    fn serde_cdungeon() {
        let byml = load_cdungeon_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let data = Byml::from(munt.clone()).to_binary(roead::Endian::Big);
        let byml2 = Byml::from_binary(&data).unwrap();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        assert_eq!(munt, munt2);
    }

    #[test]
    fn diff_mainfield() {
        let byml = load_mainfield_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let byml2 = load_mod_mainfield_munt();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        let _diff = munt.diff(&munt2);
    }

    #[test]
    fn diff_cdungeon() {
        let byml = load_cdungeon_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let byml2 = load_mod_cdungeon_munt();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        let _diff = munt.diff(&munt2);
    }

    #[test]
    fn merge_mainfield() {
        let byml = load_mainfield_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let byml2 = load_mod_mainfield_munt();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        let diff = munt.diff(&munt2);
        let merged = munt.merge(&diff);
        assert_eq!(merged, munt2);
    }

    #[test]
    fn merge_cdungeon() {
        let byml = load_cdungeon_munt();
        let munt = super::MapUnit::try_from(&byml).unwrap();
        let byml2 = load_cdungeon_munt();
        let munt2 = super::MapUnit::try_from(&byml2).unwrap();
        let diff = munt.diff(&munt2);
        let merged = munt.merge(&diff);
        assert_eq!(merged, munt2);
    }
}