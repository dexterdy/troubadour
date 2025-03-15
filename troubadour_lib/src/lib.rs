use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

use error::{convert_read_file_error, convert_write_file_error, Error, ErrorVariant};
use indexmap::{IndexMap, IndexSet};
use player::{Player, Serializable};
use serde::{Deserialize, Serialize};

mod error;
pub mod player;

#[derive(Serialize, Deserialize)]
struct SerializableAppState {
    players: HashMap<String, Serializable>,
    top_group: IndexSet<String>,
    groups: IndexMap<String, IndexSet<String>>,
}

pub fn save(
    players: &HashMap<String, Player>,
    top_group: &IndexSet<String>,
    groups: &IndexMap<String, IndexSet<String>>,
    path: &Path,
) -> Result<(), Error> {
    let serializable: HashMap<String, Serializable> = players
        .iter()
        .map(|(k, p)| (k.clone(), p.to_serializable()))
        .collect();
    let ser_app_self = SerializableAppState {
        players: serializable,
        top_group: top_group.clone(),
        groups: groups.clone(),
    };
    let json = serde_json::to_string(&ser_app_self).map_err(|e| Error {
        msg: "error: could not serialize to json. This is a bug. Contact the developer".to_string(),
        variant: ErrorVariant::Serialization,
        source: Some(e.into()),
    })?;
    fs::write(path, json).map_err(|e| convert_write_file_error(path, e, error::FileKind::Save))?;
    Ok(())
}

pub fn load(
    path: &Path,
) -> Result<
    (
        HashMap<String, Player>,
        IndexSet<String>,
        IndexMap<String, IndexSet<String>>,
    ),
    Error,
> {
    let json: SerializableAppState = serde_json::from_reader(
        File::open(path).map_err(|e| convert_read_file_error(path, e, error::FileKind::Save))?,
    )
    .map_err(|e| Error {
        msg: "error: could not deserialize from json. This is a bug. Contact the developer"
            .to_string(),
        variant: ErrorVariant::Deserialization,
        source: Some(e.into()),
    })?;

    let mut players = HashMap::new();
    let mut top_group = IndexSet::new();
    let mut groups = IndexMap::new();

    let mut handle_new_player = |name: String, group: &mut IndexSet<String>| -> Result<(), Error> {
        let player = json.players.get(&name).unwrap();

        players.insert(name.clone(), Player::from_serializable(player)?);

        group.insert(name.clone());

        Ok(())
    };

    for name in json.top_group {
        handle_new_player(name, &mut top_group)?;
    }

    for (group_name, group) in json.groups {
        let mut new_group = IndexSet::new();

        for name in group {
            handle_new_player(name, &mut new_group)?;
        }

        groups.insert(group_name.clone(), new_group);
    }

    Ok((players, top_group, groups))
}
