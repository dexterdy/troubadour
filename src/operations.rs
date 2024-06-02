use anyhow::Error;
use indexmap::{IndexMap, IndexSet};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::player::Player;
use crate::player::Serializable;
use crate::{get_confirmation, get_option, readline, AppState};

fn validate_selection(
    state: &AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
) -> Result<(), Error> {
    for group_id in group_ids {
        if !state.groups.contains_key(group_id) {
            return Err(Error::msg(format!(
                "error: no group found with name {}",
                group_id
            )));
        }
    }
    if ids.len() == 1 && ids[0].to_lowercase() == "all" {
        return Ok(());
    }
    for id in ids {
        if id.to_lowercase() == "all" {
            return Err(Error::msg(
                "error: id 'all' is only valid when no other id's are specified",
            ));
        }

        if !state.players.contains_key(id) {
            return Err(Error::msg(format!(
                "error: no player found with name {}",
                id
            )));
        }
    }
    if state.top_group.len() == 0 {
        return Err(Error::msg(
            "error: no players to select. Add a player first",
        ));
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

        if ids.len() == 0 && group_ids.len() == 0 && state.top_group.len() > 0 {
            add_id(state.top_group.last().ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?);
        }
    }

    for id in selection {
        callback(state.players.get_mut(&id).unwrap())?;
    }
    Ok(())
}

fn show_selection(
    state: &AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
) -> Result<(), Error> {
    validate_selection(state, ids, group_ids)?;
    if ids.len() == 1 && ids[0].to_lowercase() == "all" {
        for id in &state.top_group {
            println!("{}", state.players.get(id).ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?.to_string());
        }
    } else {
        for id in ids {
            println!("{}", state.players.get(id).ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?.to_string());
        }
    }
    for group_id in group_ids {
        println!("\n{}\n", group_id);
        for id in state.groups.get(group_id).unwrap() {
            println!("{}", state.players.get(id).ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?.to_string());
        }
    }
    if ids.len() == 0 && group_ids.len() == 0 && state.top_group.len() > 0 {
        println!("{}", state.players.get(state.top_group.last().unwrap()).ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?.to_string());
    }
    Ok(())
}

pub struct RespondResult {
    pub mutated: bool,
    pub saved: bool,
    pub quit: bool,
}

pub fn add(state: &mut AppState, path: PathBuf, name: String) -> Result<RespondResult, Error> {
    if &name.to_lowercase() == "all" {
        return Err(Error::msg(
            "error: you cannot use the name 'all', because it is a keyword.",
        ));
    }
    if state.players.contains_key(&name) {
        return Err(Error::msg(format!(
            "error: you cannot use the name '{name}', because it is already used."
        )));
    }
    let new_player = Player::new(path, name.clone())?;
    println!("{}", new_player.to_string());
    state.players.insert(name.clone(), new_player);
    state.top_group.insert(name);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn remove(state: &mut AppState, ids: Vec<String>) -> Result<RespondResult, Error> {
    validate_selection(state, &ids, &vec![])?;
    if ids.len() == 0 {
        return Err(Error::msg(
            "error: please provide the ids of the players that you want to remove",
        ));
    }
    for id in &ids {
        if id.to_lowercase() == "all" {
            return Err(Error::msg(
                "error: 'all' is not a valid id for this command",
            ));
        }
    }
    if get_confirmation("Are you sure you want to remove these players?")? {
        println!("Removed {}", ids.join(", "));
        state.players.retain(|k, _| !ids.contains(k));
        state.top_group.retain(|n| !ids.contains(n));
        for (_, group) in &mut state.groups {
            group.retain(|n| !ids.contains(n));
        }
        Ok(RespondResult {
            mutated: true,
            saved: false,
            quit: false,
        })
    } else {
        Ok(RespondResult {
            mutated: false,
            saved: false,
            quit: false,
        })
    }
}

pub fn play(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| p.play())?;
    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn stop(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| Ok(p.stop()))?;
    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn pause(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| Ok(p.pause()))?;
    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn set_volume(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
    volume: u32,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| Ok(p.volume(volume)))?;
    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn show(
    state: &AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
) -> Result<RespondResult, Error> {
    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn toggle_loop(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| {
        p.toggle_loop(true);
        p.loop_length(duration);
        p.apply_settings_in_place(false)?;
        Ok(())
    })?;

    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}
pub fn unloop(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| {
        p.toggle_loop(false);
        p.apply_settings_in_place(false)?;
        Ok(())
    })?;

    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_start(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| {
        p.skip_duration(duration);
        p.apply_settings_in_place(false)?;
        Ok(())
    })?;

    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_end(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| {
        p.take_duration(duration);
        p.apply_settings_in_place(false)?;
        Ok(())
    })?;

    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn delay(
    state: &mut AppState,
    ids: Vec<String>,
    group_ids: Vec<String>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    apply_selection(state, &ids, &group_ids, |p| {
        p.set_delay(duration);
        p.apply_settings_in_place(false)?;
        Ok(())
    })?;

    show_selection(state, &ids, &group_ids)?;
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn group(state: &mut AppState, name: String, ids: Vec<String>) -> Result<RespondResult, Error> {
    validate_selection(state, &ids, &vec![])?;
    if state.groups.contains_key(&name) {
        let group = state.groups.get_mut(&name).unwrap();
        group.extend(ids);
    } else {
        let mut group = IndexSet::new();
        group.extend(ids);
        state.groups.insert(name, group);
    };
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn ungroup(
    state: &mut AppState,
    name: String,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    validate_selection(state, &ids, &vec![name.clone()])?;
    let group = state.groups.get_mut(&name).unwrap();
    for id in &ids {
        if !group.contains(id) {
            return Err(Error::msg(format!(
                "error: {id} is not part of the group {name}"
            )));
        }
    }
    let ids: IndexSet<String> = ids.into_iter().collect();
    if ids.len() == group.len() {
        state.groups.shift_remove(&name);
    } else {
        for id in ids {
            group.shift_remove(&id);
        }
    }
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

#[derive(Serialize, Deserialize)]
struct SerializableAppState {
    players: HashMap<String, Serializable>,
    top_group: IndexSet<String>,
    groups: IndexMap<String, IndexSet<String>>,
}

pub fn save(state: &mut AppState, path: &Path) -> Result<RespondResult, Error> {
    let serializable: HashMap<String, Serializable> = state
        .players
        .iter()
        .map(|(k, p)| (k.clone(), p.to_serializable()))
        .collect();
    let ser_app_state = SerializableAppState {
        players: serializable,
        top_group: state.top_group.clone(),
        groups: state.groups.clone(),
    };
    let json = serde_json::to_string(&ser_app_state)?;
    fs::write(path, json)?;
    Ok(RespondResult {
        mutated: false,
        saved: true,
        quit: false,
    })
}

pub fn load(
    state: &mut AppState,
    path: &Path,
    has_been_saved: bool,
) -> Result<RespondResult, Error> {
    let add_to_soundscape = state.players.is_empty()
        || get_confirmation("Do you want to add this to you current soundscape?")?;
    let perform_action = add_to_soundscape
        || has_been_saved
        || get_confirmation("Are you sure you want to overwrite this soundscape without saving?")?;
    if perform_action {
        let json: SerializableAppState = serde_json::from_reader(File::open(path)?)?;

        if !add_to_soundscape {
            state.players.clear();
            state.top_group.clear();
            state.groups.clear();
        }

        let get_new_name = |thing: String, name: String, existing_group: &IndexSet<&String>| {
            let mut new_name = name.clone();
            let mut skip = false;

            while existing_group.contains(&&new_name) {
                let option = get_option(
                    format!(
                        "A {thing} with the name {new_name} already exists. Overwrite(O)/Skip(S)/Rename(R)"
                    )
                    .as_str(),
                    vec!["o", "s", "r"],
                )?;
                match option.as_str() {
                    "o" => {
                        break;
                    }
                    "s" => {
                        skip = true;
                    }
                    "r" => {
                        new_name = readline("enter new name: ")?;
                    }
                    _ => {
                        return Err(Error::msg("error: non-allowed option got through validation. This is a bug. Contact the developer"));
                    }
                }
            }

            if skip {
                return Ok(None);
            }
            Ok(Some(new_name))
        };

        let mut handle_new_player =
            |name: String, group: &mut IndexSet<String>| -> Result<(), Error> {
                let new_name = get_new_name(
                    "player".to_string(),
                    name.clone(),
                    &state.players.keys().into_iter().collect(),
                )?;

                if let None = new_name {
                    return Ok(());
                }

                let player = json.players.get(&name).unwrap();

                state.players.insert(
                    new_name.clone().unwrap(),
                    Player::from_serializable(player)?,
                );

                group.insert(new_name.unwrap());

                Ok(())
            };

        for name in json.top_group {
            handle_new_player(name, &mut state.top_group)?;
        }

        for (group_name, group) in json.groups {
            let new_name = get_new_name(
                "group".to_string(),
                group_name,
                &state.groups.keys().into_iter().collect(),
            )?;

            if let None = new_name {
                continue;
            }

            let mut new_group = IndexSet::new();

            for name in group {
                handle_new_player(name, &mut new_group)?;
            }

            state.groups.insert(new_name.unwrap(), new_group);
        }

        show_selection(
            state,
            &state.top_group.clone().into_iter().collect(),
            &state.groups.keys().cloned().collect(),
        )?;
    }
    Ok(RespondResult {
        mutated: add_to_soundscape && perform_action,
        saved: !add_to_soundscape && perform_action,
        quit: false,
    })
}

pub fn exit() -> Result<RespondResult, Error> {
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: true,
    })
}
