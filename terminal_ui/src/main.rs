use anyhow::Error;
use clap::Parser;
use indexmap::{IndexMap, IndexSet};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::Duration;
use terminal_helpers::{get_confirmation, get_option, readline, show_selection};
use troubadour_lib::player::Player;
use troubadour_lib::{load, save};
use ui_definition::Commands;

// TODO: fades (fade in, fade out, fade transition, fade length with default)
// TODO: make a nice GUI
// TODO: write a bunch of tests
// TODO: copy operation
// VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

mod terminal_helpers;
mod ui_definition;

struct RespondResult {
    saved: bool,
    mutated: bool,
    quit: bool,
}

#[derive(Default)]
pub struct AppState {
    pub players: HashMap<String, Player>,
    pub top_group: IndexSet<String>,
    pub groups: IndexMap<String, IndexSet<String>>,
}

fn main() -> Result<(), String> {
    println!(
        r"Troubadour Copyright (C) 2024 J.P Hagedoorn AKA Dexterdy Krataigos
This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it
under the conditions of the GPL v3."
    );

    let mut rl = DefaultEditor::new().expect("error: could not get access to the stdin.");

    let mut state = AppState::default();

    let mut has_been_saved = true;

    loop {
        let mut should_quit = false;

        let response = readline("$", &mut rl).and_then(|line| {
            let line = line.trim();
            respond(&mut state, line, has_been_saved, &mut rl)
        });

        match response {
            Ok(RespondResult {
                saved,
                mutated,
                quit,
            }) => {
                has_been_saved = (has_been_saved && !mutated) || saved;
                should_quit = quit;
            }
            Err(err) => match err.downcast::<ReadlineError>() {
                Ok(ReadlineError::Interrupted) => should_quit = true,
                Ok(err) => println!("{err}"),
                Err(err) => println!("{err}"),
            },
        }

        if should_quit {
            let quit = has_been_saved
                || get_confirmation("Are you sure you want to exit without saving?", &mut rl)
                    .unwrap_or_else(|e| {
                        matches!(
                            e.downcast::<ReadlineError>(),
                            Ok(ReadlineError::Interrupted)
                        )
                    });
            if quit {
                break Ok(());
            }
        }
    }
}

fn respond(
    state: &mut AppState,
    line: &str,
    has_been_saved: bool,
    rl: &mut Editor<(), FileHistory>,
) -> Result<RespondResult, Error> {
    if line.is_empty() {
        return Ok(RespondResult {
            saved: false,
            mutated: false,
            quit: false,
        });
    }
    let args = shlex::split(line).ok_or_else(|| {
        Error::msg("error: cannot parse input. Perhaps you have erroneous quotation(\"\")?")
    })?;
    let matches = Commands::try_parse_from(args)?;
    match matches {
        Commands::Add { path, name } => {
            if &name.to_lowercase() == "all" {
                return Err(Error::msg(
                    "error: you cannot use the name 'all', because it is a keyword.".to_string(),
                ));
            }
            if state.players.contains_key(&name) {
                return Err(Error::msg(format!(
                    "error: you cannot use the name '{name}', because it is already used."
                )));
            }
            let new_player = Player::new(path, name.clone())?;
            state.players.insert(name.clone(), new_player);
            state.top_group.insert(name.clone());
            show_selection(state, &vec![name], &vec![])?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Remove { ids } => {
            let confirmation = get_confirmation(
                "Are you sure you want to delete these players and/or groups?",
                rl,
            )?;
            if confirmation {
                validate_selection(state, &ids, &vec![])?;
                if ids.is_empty() {
                    return Err(Error::msg(
                        "error: please provide the ids of the players that you want to remove"
                            .to_string(),
                    ));
                }
                for id in &ids {
                    if id.to_lowercase() == "all" {
                        return Err(Error::msg(
                            "error: 'all' is not a valid id for this command".to_string(),
                        ));
                    }
                }
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
                    saved: false,
                    mutated: false,
                    quit: false,
                })
            }
        }
        Commands::Copy { ids, groups } => {
            validate_selection(state, &ids, &groups)?;

            let get_new_player = |state: &AppState, id: String| {
                let p = state.players.get(&id).unwrap();
                let mut new_name = format!("{}(2)", p.name).clone();
                let mut copy_number = 3;
                while state.players.contains_key(&new_name) {
                    new_name = format!("{}({})", p.name, copy_number);
                    copy_number += 1;
                }
                p.copy(&new_name)
            };

            let mut unique_groups = groups.clone();
            unique_groups.sort();
            unique_groups.dedup();

            for group_id in unique_groups {
                let mut new_name = format!("{}(2)", group_id).clone();
                let mut copy_number = 3;
                while state.groups.contains_key(&new_name) {
                    new_name = format!("{}({})", group_id, copy_number);
                    copy_number += 1;
                }
                let group_players = state.groups.get(&group_id).unwrap();
                let mut new_group_players = IndexSet::new();
                for p_id in group_players {
                    let mut new_p = get_new_player(state, p_id.clone())?;
                    new_group_players.insert(new_p.name.clone());
                    new_p.group = Some(new_name.clone());
                    state.players.insert(new_p.name.clone(), new_p);
                }
                state.groups.insert(new_name, new_group_players);
            }

            let mut unique_ids = ids.clone();
            unique_ids.sort();
            unique_ids.dedup();

            if ids.is_empty() && groups.is_empty() && !state.top_group.is_empty() {
                unique_ids.push(state.top_group.last().unwrap().clone());
            }

            for id in unique_ids {
                let new_p = get_new_player(&state, id)?;
                let part_of_group = new_p
                    .group
                    .clone()
                    .and_then(|group_name| state.groups.get_mut(&group_name))
                    .map(|group| group.insert(new_p.name.clone()))
                    .is_some();
                if !part_of_group {
                    state.top_group.insert(new_p.name.clone());
                }
                state.players.insert(new_p.name.clone(), new_p);
            }

            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Play { ids, groups } => {
            apply_selection(state, &ids, &groups, |p| Ok(p.play()?))?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: false,
                saved: false,
                quit: false,
            })
        }
        Commands::Stop { ids, groups } => {
            apply_selection(state, &ids, &groups, |p| {
                p.stop();
                Ok(())
            })?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: false,
                saved: false,
                quit: false,
            })
        }
        Commands::Pause { ids, groups } => {
            apply_selection(state, &ids, &groups, |p| {
                p.pause();
                Ok(())
            })?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: false,
                saved: false,
                quit: false,
            })
        }
        Commands::Volume {
            ids,
            groups,
            volume,
        } => {
            apply_selection(state, &ids, &groups, |p| {
                p.volume(volume);
                Ok(())
            })?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Show { ids, groups } => {
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                saved: false,
                mutated: false,
                quit: false,
            })
        }
        Commands::Loop {
            ids,
            groups,
            duration,
        } => {
            apply_selection(state, &ids, &groups, |p| {
                Ok(p.toggle_loop(true, duration.unwrap_or(Duration::from_secs(0)))?)
            })?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Unloop { ids, groups } => {
            apply_selection(state, &ids, &groups, |p| {
                Ok(p.toggle_loop(false, p.loop_gap)?)
            })?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::CutStart {
            ids,
            groups,
            duration,
        } => {
            apply_selection(state, &ids, &groups, |p| Ok(p.cut_start(duration)?))?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::CutEnd {
            ids,
            groups,
            duration,
        } => {
            apply_selection(state, &ids, &groups, |p| Ok(p.cut_end(duration)?))?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Delay {
            ids,
            groups,
            duration,
        } => {
            apply_selection(state, &ids, &groups, |p| Ok(p.set_delay(duration)?))?;
            show_selection(state, &ids, &groups)?;
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Group { group: name, ids } => {
            validate_selection(state, &ids, &vec![])?;
            for id in &ids {
                state.top_group.shift_remove(id);
                let player = state.players.get_mut(id).unwrap();
                if let Some(group) = &player.group {
                    state
                    .groups
                    .get_mut(group)
                    .ok_or(Error::msg("error: player carries reference to non-existent group. This is a bug. Contact the developer".to_string()))?
                    .shift_remove(id);
                }
                player.group = Some(name.clone());
            }
            if state.groups.contains_key(&name) {
                let group = state.groups.get_mut(&name).unwrap();
                group.extend(ids.clone());
            } else {
                let mut group = IndexSet::new();
                group.extend(ids.clone());
                state.groups.insert(name, group);
            };
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Ungroup { group: name, ids } => {
            validate_selection(state, &ids, &vec![name.clone()])?;
            let group = state.groups.get_mut(&name).unwrap();
            for id in &ids {
                if !group.contains(id) {
                    return Err(Error::msg(format!(
                        "error: {id} is not part of the group {name}"
                    )));
                }
            }
            let ids: IndexSet<String> = ids.clone().into_iter().collect();
            if ids.len() == group.len() {
                state.groups.shift_remove(&name);
            } else {
                for id in &ids {
                    group.shift_remove(id);
                }
            }
            for id in &ids {
                let player = state.players.get_mut(id).unwrap();
                player.group = None;
                state.top_group.insert(id.clone());
            }
            Ok(RespondResult {
                mutated: true,
                saved: false,
                quit: false,
            })
        }
        Commands::Save { path } => {
            save(&state.players, &state.top_group, &state.groups, &path)?;
            Ok(RespondResult {
                saved: true,
                mutated: false,
                quit: false,
            })
        }
        Commands::Load { path } => load_combine_or_overwrite(state, path, has_been_saved, rl),
        Commands::Exit => Ok(RespondResult {
            saved: false,
            mutated: false,
            quit: true,
        }),
    }
}

fn load_combine_or_overwrite(
    state: &mut AppState,
    path: PathBuf,
    has_been_saved: bool,
    rl: &mut Editor<(), FileHistory>,
) -> Result<RespondResult, Error> {
    let is_empty =
        state.players.is_empty() && state.groups.is_empty() && state.top_group.is_empty();
    let overwrite = is_empty
        || get_option(
            "Do you want to combine soundscapes?",
            vec![("Combine", "c"), ("Overwrite", "o")],
            rl,
        )? == "o";

    if overwrite {
        let confirmation = has_been_saved
            || get_confirmation(
                "You have unsaved changes. Are you sure you want to overwrite?",
                rl,
            )?;

        return if confirmation {
            let new = load(&path)?;
            state.players = new.0;
            state.top_group = new.1;
            state.groups = new.2;
            Ok(RespondResult {
                saved: true,
                mutated: true,
                quit: false,
            })
        } else {
            Ok(RespondResult {
                saved: false,
                mutated: false,
                quit: false,
            })
        };
    }

    let mut new = load(&path)?;

    macro_rules! get_changes_helper {
        ($item:expr, $new_map:expr, $map:expr, $options:expr) => {{
            let mut renames = vec![];
            let mut to_skip = vec![];

            for name in $new_map.keys() {
                if $map.contains_key(name) {
                    let option = get_option(
                        &format!("A {} by the name of {name} already exists.", $item),
                        $options,
                        rl,
                    )?;
                    match option.as_str() {
                        "m" | "o" => (),
                        "r" => {
                            let new_name: String = readline("What should the new name be?:", rl)?;
                            renames.push((name.clone(), new_name));
                        }
                        _ => {
                            to_skip.push(name.clone());
                        }
                    }
                }
            }
            (renames, to_skip)
        }};
    }

    let (player_renames, players_to_skip) = get_changes_helper!(
        "player",
        new.0,
        state.players,
        vec![("Overwrite", "o"), ("Rename", "r"), ("Skip", "s")]
    );

    let (group_renames, groups_to_skip) = get_changes_helper!(
        "group",
        new.2,
        state.groups,
        vec![("Merge", "m"), ("Rename", "r"), ("Skip", "s")]
    );

    for (name, new_name) in player_renames {
        if let Some(mut player) = new.0.remove(&name) {
            player.name = new_name.clone();
            new.0.insert(new_name.clone(), player);
        }

        if let Some((index, _)) = new.1.shift_remove_full(&name) {
            new.1.shift_insert(index, new_name.clone());
        }

        for group in new.2.values_mut() {
            let res = group.shift_remove_full(&name);
            if let Some((index, _)) = res {
                group.shift_insert(index, new_name.clone());
            }
        }
    }

    for skip in players_to_skip {
        new.0.remove(&skip);
        new.1.shift_remove(&skip);
        for group in new.2.values_mut() {
            group.shift_remove(&skip);
        }
    }

    for skip in groups_to_skip {
        new.2.shift_remove(&skip);
    }

    for (name, new_name) in group_renames {
        if let Some((index, _, group)) = new.2.shift_remove_full(&name) {
            new.2.shift_insert(index, new_name.clone(), group);
        }
    }

    state.players.extend(new.0);
    state.top_group.extend(new.1);
    for (name, new_group) in new.2 {
        if let Some(group) = state.groups.get_mut(&name) {
            group.extend(new_group);
        } else {
            state.groups.insert(name, new_group);
        }
    }

    Ok(RespondResult {
        saved: false,
        mutated: true,
        quit: false,
    })
}

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
                "error: id 'all' is only valid when no other id's are specified".to_string(),
            ));
        }

        if !state.players.contains_key(id) {
            return Err(Error::msg(format!(
                "error: no player found with name {}",
                id
            )));
        }
    }
    if state.players.is_empty() {
        return Err(Error::msg(
            "error: no players to select. Add a player first".to_string(),
        ));
    }
    Ok(())
}

fn get_selection(
    state: &AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
) -> Result<Vec<String>, Error> {
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
            add_id(state.top_group.last().ok_or(
                Error::msg(
                    "error: internal reference to player that does not exist. This is a bug. Contact the developer".to_string()
                )
            )?);
        }
    }

    Ok(selection.into_iter().collect())
}

fn apply_selection(
    state: &mut AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
    mut callback: impl FnMut(&mut Player) -> Result<(), Error>,
) -> Result<(), Error> {
    validate_selection(state, ids, group_ids)?;
    let selection = get_selection(state, ids, group_ids)?;

    for id in selection {
        callback(state.players.get_mut(&id).unwrap())?;
    }
    Ok(())
}
