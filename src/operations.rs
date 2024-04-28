use anyhow::Error;
use std::collections::HashSet;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{fs, ptr};

use crate::get_confirmation;
use crate::player::Serialisable;
use crate::{player::Player, IdOrName};

macro_rules! apply_vec {
    ($vec:expr, $($inside:tt)*) => {
        for x in $vec {
            apply_vec!(x; $($inside)*);
        }
    };
    ($x:ident; $func:ident($($arg:expr),*) $(,$($more:tt)*)?) => {
        $x.$func($($arg),*);
        $(apply_vec!($x; $($more)*))?
    };
    ($x:ident; $func:ident($($arg:expr),*) ? $(,$($more:tt)*)?) => {
        $x.$func($($arg),*)?;
        $(apply_vec!($x; $($more)*))?
    };
}

macro_rules! show_selection {
    ($selection:expr) => {
        for player in $selection {
            println!("{}", player.to_string())
        }
    };
}

pub struct RespondResult {
    pub mutated: bool,
    pub saved: bool,
    pub quit: bool,
}

fn select_players(players: &[Player], ids: Vec<IdOrName>) -> Result<Vec<&Player>, Error> {
    if players.is_empty() {
        return Err(Error::msg(
            "error: there are addcurrently no players to select",
        ));
    }
    if ids.contains(&IdOrName::All) {
        if ids[0] == IdOrName::All && ids.len() == 1 {
            return Ok(players.iter().collect());
        }
        return Err(Error::msg("'all' has to be the only id."));
    }
    if ids.is_empty() {
        return Ok(vec![players.last().unwrap()]);
    }

    let ids_set: HashSet<usize> = ids
        .into_iter()
        .map(|id| match id {
            IdOrName::Id(num) => Ok(num),
            IdOrName::Name(name) => players
                .iter()
                .position(|p| p.name == name)
                .ok_or_else(|| Error::msg(format!("error: no player found with the name {name}"))),
            IdOrName::All => unreachable!(),
        })
        .collect::<Result<HashSet<usize>, Error>>()?;

    Ok(ids_set.into_iter().map(|id| &players[id]).collect())
}

fn select_players_mut(
    players: &mut [Player],
    ids: Vec<IdOrName>,
) -> Result<Vec<&mut Player>, Error> {
    if players.is_empty() {
        return Err(Error::msg(
            "error: there are currently no players to select",
        ));
    }
    if ids.contains(&IdOrName::All) {
        if ids[0] == IdOrName::All && ids.len() == 1 {
            return Ok(players.iter_mut().collect());
        }
        return Err(Error::msg("'all' has to be the only id."));
    }
    if ids.is_empty() {
        return Ok(vec![players.last_mut().unwrap()]);
    }

    let ids_set: HashSet<usize> = ids
        .into_iter()
        .map(|id| match id {
            IdOrName::Id(num) => Ok(num),
            IdOrName::Name(name) => players
                .iter_mut()
                .position(|p| p.name == name)
                .ok_or_else(|| Error::msg(format!("error: no player found with the name {name}"))),
            IdOrName::All => unreachable!(),
        })
        .collect::<Result<HashSet<usize>, Error>>()?;

    // because of the hashset, I know that the id's will be unique
    // meaning that borrowing multiple items in players as mutable is safe
    // as long as I only borrow that ones corresponding to the ids
    Ok(ids_set
        .into_iter()
        .map(|id| unsafe { &mut *ptr::addr_of_mut!(players[id]) })
        .collect())
}

pub fn add(players: &mut Vec<Player>, path: PathBuf, name: String) -> Result<RespondResult, Error> {
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
    if players.iter_mut().filter(|p| p.name == name).count() > 0 {
        return Err(Error::msg(format!(
            "error: you cannot use the name '{name}', because it is already used."
        )));
    }
    let new_player = Player::new(path, name)?;
    println!("{}", new_player.to_string());
    players.push(new_player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn remove(players: &mut Vec<Player>, ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    // unsafe to have a mut and not mut ref at the same time
    // make sure to print before remove
    // this is an ugly hack
    let selected_players = select_players(unsafe { &*(players as *const Vec<Player>) }, ids)?
        .iter()
        .map(|p| p.name.as_str())
        .collect::<Vec<&str>>();
    if get_confirmation("Are you sure you want to remove these sounds?")? {
        println!("Removed {}", selected_players.join(", "));
        players.retain(|p| !selected_players.contains(&p.name.as_str()));
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

pub fn play(players: &mut [Player], ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(&mut player, play()?);
    show_selection!(player);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn stop(players: &mut [Player], ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(&mut player, stop());
    show_selection!(player);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn pause(players: &mut [Player], ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(&mut player, pause());
    show_selection!(player);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn set_volume(
    players: &mut [Player],
    ids: Vec<IdOrName>,
    volume: u32,
) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(&mut player, volume(volume));
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn show(players: &[Player], ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    let player = select_players(players, ids)?;
    show_selection!(player);
    Ok(RespondResult {
        mutated: false,
        saved: false,
        quit: false,
    })
}

pub fn toggle_loop(
    players: &mut [Player],
    ids: Vec<IdOrName>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(
        &mut player,
        toggle_loop(true),
        loop_length(duration),
        apply_settings_in_place(false)?
    );
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}
pub fn unloop(players: &mut [Player], ids: Vec<IdOrName>) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(
        &mut player,
        toggle_loop(false),
        apply_settings_in_place(false)?
    );
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_start(
    players: &mut [Player],
    ids: Vec<IdOrName>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(
        &mut player,
        skip_duration(duration),
        apply_settings_in_place(false)?
    );
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn set_end(
    players: &mut [Player],
    ids: Vec<IdOrName>,
    duration: Option<Duration>,
) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(
        &mut player,
        take_duration(duration),
        apply_settings_in_place(false)?
    );
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn delay(
    players: &mut [Player],
    ids: Vec<IdOrName>,
    duration: Duration,
) -> Result<RespondResult, Error> {
    let mut player = select_players_mut(players, ids)?;
    apply_vec!(
        &mut player,
        set_delay(duration),
        apply_settings_in_place(false)?
    );
    show_selection!(player);
    Ok(RespondResult {
        mutated: true,
        saved: false,
        quit: false,
    })
}

pub fn save(players: &mut [Player], path: &Path) -> Result<RespondResult, Error> {
    let serialisable: Vec<Serialisable> = players.iter_mut().map(|p| p.to_serializable()).collect();
    let json = serde_json::to_string(&serialisable)?;
    fs::write(path, json)?;
    Ok(RespondResult {
        mutated: false,
        saved: true,
        quit: false,
    })
}
pub fn load(
    players: &mut Vec<Player>,
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
            .collect::<Vec<Player>>();
        if !add_to_soundscape {
            players.clear();
        }
        players.extend(new_players);
        show_selection!(players)
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
