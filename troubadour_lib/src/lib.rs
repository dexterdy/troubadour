mod error;
pub mod player;

use std::{
    collections::HashMap,
    fs::{self, File},
    path::Path,
};

use error::{convert_read_file_error, convert_write_file_error, Error, ErrorVariant};
use indexmap::{IndexMap, IndexSet};
use player::{Player, Serializable};
use serde::{Deserialize, Serialize};

// TODO: fades (fade in, fade out, fade transition, fade length with default)

#[derive(Serialize, Deserialize)]
struct SerializableSaveState {
    players: HashMap<String, Serializable>,
    top_group: IndexSet<String>,
    groups: IndexMap<String, IndexSet<String>>,
}

pub struct SaveState<P: Into<Serializable>> {
    pub players: HashMap<String, P>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
}

pub fn save<P: Into<Serializable>>(save_state: SaveState<P>, path: &Path) -> Result<(), Error> {
    let serializable: HashMap<String, Serializable> = save_state
        .players
        .into_iter()
        .map(|(k, p)| (k.clone(), p.into()))
        .collect();
    let ser_app_self = SerializableSaveState {
        players: serializable,
        top_group: save_state.top_group,
        groups: save_state.groups,
    };
    let json = serde_json::to_string(&ser_app_self).map_err(|e| Error {
        msg: "error: could not serialize to json. This is a bug. Contact the developer".to_string(),
        variant: ErrorVariant::Serialization,
        source: Some(e.into()),
    })?;
    fs::write(path, json).map_err(|e| convert_write_file_error(path, e, error::FileKind::Save))?;
    Ok(())
}

pub fn load(path: &Path) -> Result<SaveState<Player>, Error> {
    let json: SerializableSaveState = serde_json::from_reader(
        File::open(path).map_err(|e| convert_read_file_error(path, e, error::FileKind::Save))?,
    )
    .map_err(|e| Error {
        msg: "error: could not deserialize from json. This is a bug. Contact the developer"
            .to_string(),
        variant: ErrorVariant::Deserialization,
        source: Some(e.into()),
    })?;

    let mut save_state = SaveState {
        players: HashMap::new(),
        top_group: IndexSet::new(),
        groups: IndexMap::new(),
    };

    let mut handle_new_player = |name: String, group: &mut IndexSet<String>| -> Result<(), Error> {
        let player = json.players.get(&name).unwrap();

        save_state
            .players
            .insert(name.clone(), Player::from_serializable(player)?);

        group.insert(name.clone());

        Ok(())
    };

    for name in json.top_group {
        handle_new_player(name, &mut save_state.top_group)?;
    }

    for (group_name, group) in json.groups {
        let mut new_group = IndexSet::new();

        for name in group {
            handle_new_player(name, &mut new_group)?;
        }

        save_state.groups.insert(group_name.clone(), new_group);
    }

    Ok(save_state)
}
