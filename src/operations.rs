use anyhow::Error;
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::get_confirmation;
use crate::player::Player;
use crate::player::Serialisable;

macro_rules! show_selection {
    ($selection:expr) => {
        for player in $selection {
            println!("{}", player.to_string())
        }
    };
}

fn select_player_mut<'a>(
    players: &'a mut HashMap<String, Player>,
    id: &String,
) -> Result<&'a mut Player, Error> {
    players.get_mut(id).ok_or(Error::msg(format!(
        "error: no player found with name {}",
        id
    )))
}

fn select_players<'a>(
    players: &'a HashMap<String, Player>,
    ids: &Vec<String>,
) -> Result<Vec<&'a Player>, Error> {
    let mut selected_players = vec![];
    for id in ids {
        let next_player = players.get(id).ok_or(Error::msg(format!(
            "error: no player found with name {}",
            id
        )))?;
        selected_players.push(next_player);
    }
    return Ok(selected_players);
}

pub struct RespondResult {
    pub mutated: bool,
    pub saved: bool,
    pub quit: bool,
}

pub fn add(
    players: &mut HashMap<String, Player>,
    path: PathBuf,
    name: String,
) -> Result<RespondResult, Error> {
    if &name.to_lowercase() == "all" {
        return Err(Error::msg(
            "error: you cannot use the name 'all', because it is a keyword.",
        ));
    }
    if name.parse::<u32>().is_ok() {
        return Err(Error::msg(format!(
            "error: you cannot use the name '{name}', because it is a number."
        )));
    }
    if players.iter_mut().filter(|(_, p)| p.name == name).count() > 0 {
        return Err(Error::msg(format!(
            "error: you cannot use the name '{name}', because it is already used."
        )));
    }
    let new_player = Player::new(path, name.clone())?;
    println!("{}", new_player.to_string());
    players.insert(name, new_player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn remove(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        if !players.contains_key(id) {
            return Err(Error::msg(format!(
                "error: no player found with name {}",
                id
            )));
        }
    }
    if get_confirmation("Are you sure you want to remove these sounds?")? {
        println!("Removed {}", ids.join(", "));
        players.retain(|k, _| !ids.contains(k));
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
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.play()?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn stop(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.stop();
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn pause(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.pause();
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn set_volume(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
    volume: u32,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.volume(volume);
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn show(players: &HashMap<String, Player>, ids: Vec<String>) -> Result<RespondResult, Error> {
    let players = select_players(players, &ids)?;
    show_selection!(players);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn toggle_loop(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.toggle_loop(true);
        player.loop_length(duration);
        player.apply_settings_in_place(false)?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}
pub fn unloop(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.toggle_loop(false);
        player.apply_settings_in_place(false)?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_start(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.skip_duration(duration);
        player.apply_settings_in_place(false)?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_end(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.take_duration(duration);
        player.apply_settings_in_place(false)?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn delay(
    players: &mut HashMap<String, Player>,
    ids: Vec<String>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    for id in &ids {
        let player = select_player_mut(players, id)?;
        player.set_delay(duration);
        player.apply_settings_in_place(false)?;
    }
    show_selection!(select_players(players, &ids)?);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

// pub fn group(
//     players: &HashMap<String, Player>,
//     name: String,
//     ids: Vec<String>,
//     groups: &mut HashMap<String, Vec<String>>,
// ) -> Result<RespondResult, Error> {
//     select_players(players, &ids)?; // perform selection just to be sure that the players actually exist
// }

pub fn save(players: &mut HashMap<String, Player>, path: &Path) -> Result<RespondResult, Error> {
    let serialisable: Vec<Serialisable> =
        players.values_mut().map(|p| p.to_serializable()).collect();
    let json = serde_json::to_string(&serialisable)?;
    fs::write(path, json)?;
    Ok(RespondResult {
        mutated: false,
        saved: true,
        quit: false,
    })
}
pub fn load(
    players: &mut HashMap<String, Player>,
    path: &Path,
    has_been_saved: bool,
) -> Result<RespondResult, Error> {
    let add_to_soundscape = players.is_empty()
        || get_confirmation("Do you want to add this to you current soundscape?")?;
    let perform_action = add_to_soundscape
        || has_been_saved
        || get_confirmation("Are you sure you want to exit without saving?")?;
    if perform_action {
        let json: Vec<Serialisable> = serde_json::from_reader(File::open(path)?)?;
        let new_players = json
            .into_iter()
            .filter_map(|p| Player::from_serializable(&p).ok())
            .map(|p| (p.name.clone(), p))
            .collect::<HashMap<String, Player>>();
        if !add_to_soundscape {
            players.clear();
        }
        players.extend(new_players);
        show_selection!(players.values())
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
