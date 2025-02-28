use anyhow::Error;
use duration_human::DurationHuman;
use fomat_macros::fomat;
use indexmap::{IndexMap, IndexSet};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::Editor;
use std::time::Duration;
use troubadour_lib::player::Player;
use troubadour_lib::AppState;

pub fn readline(prompt: &str, rl: &mut Editor<(), FileHistory>) -> Result<String, Error> {
    let line = rl.readline(prompt);
    match line {
        Ok(line) => {
            rl.add_history_entry(line.as_str()).unwrap_or_default();
            Ok(line)
        }
        Err(ReadlineError::Eof) => Err(Error::msg("error: unexpected EOF.")),
        Err(ReadlineError::WindowResized) => readline(prompt, rl),
        Err(ReadlineError::Interrupted) => Ok(line?),
        _ => Err(Error::msg("error: could not read from stdin")),
    }
}

pub fn get_confirmation(prompt: &str, rl: &mut Editor<(), FileHistory>) -> Result<bool, Error> {
    let mut result = None;

    while result.is_none() {
        let response = readline(&format!("{prompt} Y/N: "), rl)
            .map_err(Error::msg)?
            .trim()
            .to_lowercase();

        if response.to_lowercase() != "y" && response.to_lowercase() != "n" {
            println!("{} is not a valid valid answer.", response);
            continue;
        }
        result = Some(response.to_lowercase() == "y")
    }
    Ok(result.unwrap())
}

pub fn get_option(
    prompt: &str,
    valid_options: Vec<&str>,
    rl: &mut Editor<(), FileHistory>,
) -> Result<String, Error> {
    let mut result = None;

    while result.is_none() {
        let response = readline(&format!("{prompt}: "), rl)
            .map_err(Error::msg)?
            .trim()
            .to_lowercase();

        if !valid_options.contains(&response.as_str()) {
            println!("{} is not a valid valid answer.", response);
            continue;
        }
        result = Some(response);
    }
    Ok(result.unwrap())
}

pub fn duration_to_string(dur: Duration, no_smaller_than_secs: bool) -> String {
    let nanos = if no_smaller_than_secs {
        dur.as_secs() * 1_000_000_000
    } else {
        dur.as_nanos() as u64
    };
    if nanos == 0 {
        "0s".to_string()
    } else {
        format!("{:#}", DurationHuman::from(nanos))
    }
}

pub fn player_to_string(player: &Player) -> String {
    fomat!(
        (player.name) ":"
        if player.get_is_playing() {
            "\n\tplaying"
        } else {
            if player.get_is_paused() {
                "\n\tpaused"
            } else {
                "\n\tnot playing"
            }
        }
        if player.get_is_playing() || player.get_is_paused() {
            "\n\thas been playing for: " (duration_to_string(player.get_play_time(), true))
        }
        "\n\tvolume: " (player.volume) "%"
        if player.looping {
            "\n\tloops"
            if let Some(length) = player.loop_length {
                ": every " (duration_to_string(length, false))
            }
        }
        if player.skip_length > Duration::new(0, 0) {
            "\n\tstarts at: " (duration_to_string(player.skip_length, false))
        }
        if let Some(length) = player.take_length {
            if length > Duration::new(0, 0) {
                "\n\tends at: " (duration_to_string(length, false))
            }
        }
        if player.delay_length > Duration::new(0, 0) {
            "\n\tdelay: "  (duration_to_string(player.delay_length, false))
        }
    )
}

pub fn show_selection(
    state: &AppState,
    ids: &Vec<String>,
    group_ids: &Vec<String>,
) -> Result<(), Error> {
    let mut selected_top_group = IndexSet::new();
    let mut selected_groups = IndexMap::new();
    if ids.len() == 1 && ids[0].to_lowercase() == "all" {
        selected_top_group.extend(&state.top_group);
        selected_groups.extend(
            state
                .groups
                .iter()
                .map(|(k, v)| (k, v.iter().collect()))
                .collect::<IndexMap<&String, IndexSet<&String>>>(),
        );
    } else {
        for id in ids {
            let player = state.players.get(id).unwrap();
            if let Some(group_name) = &player.group {
                if let Some(group) = selected_groups.get_mut(group_name) {
                    group.insert(id);
                } else {
                    let mut new_group = IndexSet::new();
                    new_group.insert(id);
                    selected_groups.insert(group_name, new_group);
                }
            } else {
                selected_top_group.insert(id);
            }
        }
        for group_id in group_ids {
            selected_groups.insert(
                group_id,
                state.groups.get(group_id).unwrap().iter().collect(),
            );
        }
    }
    let print_player = |id: &String| -> Result<(), Error> {
        println!("{}", player_to_string(state.players.get(id).ok_or(Error::msg("error: internal reference to player that does not exist. This is a bug. Contact the developer"))?));
        Ok(())
    };
    for id in selected_top_group {
        print_player(id)?;
    }
    for (group_name, group) in selected_groups {
        println!("\n{}\n", group_name);
        for id in group {
            print_player(id)?;
        }
    }
    if ids.is_empty() && group_ids.is_empty() && !state.top_group.is_empty() {
        print_player(state.top_group.last().unwrap())?;
    }
    Ok(())
}
