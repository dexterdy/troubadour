use crate::error::Error;
use error::{convert_read_file_error, convert_write_file_error, ErrorVariant};
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

pub mod error;
pub mod player;

use crate::player::Player;
use crate::player::Serializable;

#[derive(Default)]
pub struct AppState {
    pub players: HashMap<String, Player>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
}

pub struct RespondResult {
    pub mutated: bool,
    pub saved: bool,
}

#[derive(Serialize, Deserialize)]
struct SerializableAppself {
    players: HashMap<String, Serializable>,
    top_group: IndexSet<String>,
    groups: IndexMap<String, IndexSet<String>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState::default()
    }

    pub fn add(&mut self, path: PathBuf, name: String) -> Result<RespondResult, Error> {
        if &name.to_lowercase() == "all" {
            return Err(Error {
                msg: "error: you cannot use the name 'all', because it is a keyword.".to_string(),
                variant: ErrorVariant::NameConflict,
                source: None,
            });
        }
        if self.players.contains_key(&name) {
            return Err(Error {
                msg: format!(
                    "error: you cannot use the name '{name}', because it is already used."
                ),
                variant: ErrorVariant::NameConflict,
                source: None,
            });
        }
        let new_player = Player::new(path, name.clone())?;
        self.players.insert(name.clone(), new_player);
        self.top_group.insert(name);
        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn remove(&mut self, ids: &Vec<String>) -> Result<RespondResult, Error> {
        validate_selection(self, ids, &vec![])?;
        if ids.is_empty() {
            return Err(Error {
                msg: "error: please provide the ids of the players that you want to remove"
                    .to_string(),
                variant: ErrorVariant::MissingId,
                source: None,
            });
        }
        for id in ids {
            if id.to_lowercase() == "all" {
                return Err(Error {
                    msg: "error: 'all' is not a valid id for this command".to_string(),
                    variant: ErrorVariant::InvalidId,
                    source: None,
                });
            }
        }
        self.players.retain(|k, _| !ids.contains(k));
        self.top_group.retain(|n| !ids.contains(n));
        for (_, group) in &mut self.groups {
            group.retain(|n| !ids.contains(n));
        }
        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn play(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| p.play())?;
        Ok(RespondResult {
            mutated: false,
            saved: false,
        })
    }

    pub fn stop(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.stop();
            Ok(())
        })?;
        Ok(RespondResult {
            mutated: false,
            saved: false,
        })
    }

    pub fn pause(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.pause();
            Ok(())
        })?;
        Ok(RespondResult {
            mutated: false,
            saved: false,
        })
    }

    pub fn set_volume(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
        volume: u32,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.volume(volume);
            Ok(())
        })?;
        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn toggle_loop(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
        duration: Option<Duration>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.toggle_loop(true);
            p.loop_length(duration);
            p.apply_settings_in_place(false)?;
            Ok(())
        })?;

        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }
    pub fn unloop(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.toggle_loop(false);
            p.apply_settings_in_place(false)?;
            Ok(())
        })?;

        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn set_start(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
        duration: Duration,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.skip_duration(duration);
            p.apply_settings_in_place(false)?;
            Ok(())
        })?;

        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn set_end(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
        duration: Option<Duration>,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.take_duration(duration);
            p.apply_settings_in_place(false)?;
            Ok(())
        })?;

        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn delay(
        &mut self,
        ids: &Vec<String>,
        group_ids: &Vec<String>,
        duration: Duration,
    ) -> Result<RespondResult, Error> {
        apply_selection(self, ids, group_ids, |p| {
            p.set_delay(duration);
            p.apply_settings_in_place(false)?;
            Ok(())
        })?;

        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn group(&mut self, name: String, ids: &Vec<String>) -> Result<RespondResult, Error> {
        validate_selection(self, ids, &vec![])?;
        for id in ids {
            self.top_group.shift_remove(id);
            let player = self.players.get_mut(id).unwrap();
            if let Some(group) = &player.group {
                self
                    .groups
                    .get_mut(group)
                    .ok_or(Error {
                        msg:"error: player carries reference to non-existent group. This is a bug. Contact the developer".to_string(),
                        variant: ErrorVariant::InvalidGroupId,
                        source: None
                    })?
                    .shift_remove(id);
            }
            player.group = Some(name.clone());
        }
        if self.groups.contains_key(&name) {
            let group = self.groups.get_mut(&name).unwrap();
            group.extend(ids.clone());
        } else {
            let mut group = IndexSet::new();
            group.extend(ids.clone());
            self.groups.insert(name, group);
        };
        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn ungroup(&mut self, name: String, ids: &Vec<String>) -> Result<RespondResult, Error> {
        validate_selection(self, ids, &vec![name.clone()])?;
        let group = self.groups.get_mut(&name).unwrap();
        for id in ids {
            if !group.contains(id) {
                return Err(Error {
                    msg: format!("error: {id} is not part of the group {name}"),
                    variant: ErrorVariant::InvalidId,
                    source: None,
                });
            }
        }
        let ids: IndexSet<String> = ids.clone().into_iter().collect();
        if ids.len() == group.len() {
            self.groups.shift_remove(&name);
        } else {
            for id in &ids {
                group.shift_remove(id);
            }
        }
        for id in &ids {
            let player = self.players.get_mut(id).unwrap();
            player.group = None;
            self.top_group.insert(id.clone());
        }
        Ok(RespondResult {
            mutated: true,
            saved: false,
        })
    }

    pub fn save(&mut self, path: &Path) -> Result<RespondResult, Error> {
        let serializable: HashMap<String, Serializable> = self
            .players
            .iter()
            .map(|(k, p)| (k.clone(), p.to_serializable()))
            .collect();
        let ser_app_self = SerializableAppself {
            players: serializable,
            top_group: self.top_group.clone(),
            groups: self.groups.clone(),
        };
        let json = serde_json::to_string(&ser_app_self).map_err(|e| Error {
            msg: "error: could not serialize to json. This is a bug. Contact the developer"
                .to_string(),
            variant: ErrorVariant::Serialization,
            source: Some(e.into()),
        })?;
        fs::write(path, json)
            .map_err(|e| convert_write_file_error(path, e, error::FileKind::Save))?;
        Ok(RespondResult {
            mutated: false,
            saved: true,
        })
    }

    pub fn load(path: &Path) -> Result<AppState, Error> {
        let json: SerializableAppself = serde_json::from_reader(
            File::open(path)
                .map_err(|e| convert_read_file_error(path, e, error::FileKind::Save))?,
        )
        .map_err(|e| Error {
            msg: "error: could not deserialize from json. This is a bug. Contact the developer"
                .to_string(),
            variant: ErrorVariant::Deserialization,
            source: Some(e.into()),
        })?;

        let mut new = Self::new();

        let mut handle_new_player =
            |name: String, group: &mut IndexSet<String>| -> Result<(), Error> {
                let player = json.players.get(&name).unwrap();

                new.players
                    .insert(name.clone(), Player::from_serializable(player)?);

                group.insert(name.clone());

                Ok(())
            };

        for name in json.top_group {
            handle_new_player(name, &mut new.top_group)?;
        }

        for (group_name, group) in json.groups {
            let mut new_group = IndexSet::new();

            for name in group {
                handle_new_player(name, &mut new_group)?;
            }

            new.groups.insert(group_name.clone(), new_group);
        }

        Ok(new)
    }
}

fn validate_selection(
    state: &AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
) -> Result<(), Error> {
    for group_id in group_ids {
        if !state.groups.contains_key(group_id) {
            return Err(Error {
                msg: format!("error: no group found with name {}", group_id),
                variant: ErrorVariant::InvalidGroupId,
                source: None,
            });
        }
    }
    if ids.len() == 1 && ids[0].to_lowercase() == "all" {
        return Ok(());
    }
    for id in ids {
        if id.to_lowercase() == "all" {
            return Err(Error {
                msg: "error: id 'all' is only valid when no other id's are specified".to_string(),
                variant: ErrorVariant::InvalidId,
                source: None,
            });
        }

        if !state.players.contains_key(id) {
            return Err(Error {
                msg: format!("error: no player found with name {}", id),
                variant: ErrorVariant::InvalidId,
                source: None,
            });
        }
    }
    if state.players.is_empty() {
        return Err(Error {
            msg: "error: no players to select. Add a player first".to_string(),
            variant: ErrorVariant::NoPlayers,
            source: None,
        });
    }
    Ok(())
}

fn apply_selection(
    state: &mut AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
    callback: impl Fn(&mut Player) -> Result<(), Error>,
) -> Result<(), Error> {
    validate_selection(state, ids, group_ids)?;
    let mut selection = HashSet::new();

    if ids.len() == 1 && ids[0].to_lowercase() == "all" {
        selection.extend(state.top_group.clone());
    } else {
        let mut add_id = |id: &String| selection.insert(id.clone());

        for id in ids {
            add_id(id);
        }

        for group_id in group_ids {
            for id in state.groups.get(group_id).unwrap() {
                add_id(id);
            }
        }

        if ids.is_empty() && group_ids.is_empty() && !state.top_group.is_empty() {
            add_id(state.top_group.last().ok_or(Error {
                msg:"error: internal reference to player that does not exist. This is a bug. Contact the developer".to_string(),
                variant: ErrorVariant::InvalidId,
                source: None
            })?);
        }
    }

    for id in selection {
        callback(state.players.get_mut(&id).unwrap())?;
    }
    Ok(())
}
