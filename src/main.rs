use anyhow::Error;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};
use std::path::PathBuf;
use terminal_helpers::{get_confirmation, get_option, readline, show_selection};
use troubadour_lib::{AppState, RespondResult};
use ui_definition::Commands;

// TODO: Implement a sound length feature, based on amount samples
// TODO: fades (fade in, fade out, fade transition, fade length with default)
// TODO: macros (sets of commands that you can give a name)
// TODO: make a nice GUI
// TODO: write a bunch of tests
// TODO: copy operation
// VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

mod terminal_helpers;
mod ui_definition;

struct InternalRespondResult {
    saved: bool,
    mutated: bool,
    quit: bool,
}

fn main() -> Result<(), String> {
    println!(
        r"Troubadour Copyright (C) 2024 J.P Hagedoorn AKA Dexterdy Krataigos
This program comes with ABSOLUTELY NO WARRANTY.
This is free software, and you are welcome to redistribute it
under the conditions of the GPL v3."
    );

    let mut rl = DefaultEditor::new().expect("error: could not get access to the stdin.");

    let mut state = AppState::new();

    let mut has_been_saved = true;

    loop {
        let mut should_quit = false;

        let response = readline("$", &mut rl).and_then(|line| {
            let line = line.trim();
            respond(&mut state, line, has_been_saved, &mut rl)
        });

        match response {
            Ok(InternalRespondResult {
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
) -> Result<InternalRespondResult, Error> {
    if line.is_empty() {
        return Ok(InternalRespondResult {
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
            let res = Ok(to_internal(state.add(path, name.clone())?));
            show_selection(state, &vec![name], &vec![])?;
            res
        }
        Commands::Remove { ids } => {
            let confirmation = get_confirmation(
                "Are you sure you want to delete these players and/or groups?",
                rl,
            )?;
            if confirmation {
                Ok(to_internal(state.remove(&ids)?))
            } else {
                Ok(InternalRespondResult {
                    saved: false,
                    mutated: false,
                    quit: false,
                })
            }
        }
        Commands::Play { ids, groups } => {
            let res = Ok(to_internal(state.play(&ids, &groups)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Stop { ids, groups } => {
            let res = Ok(to_internal(state.stop(&ids, &groups)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Pause { ids, groups } => {
            let res = Ok(to_internal(state.pause(&ids, &groups)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Volume {
            ids,
            groups,
            volume,
        } => {
            let res = Ok(to_internal(state.set_volume(&ids, &groups, volume)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Show { ids, groups } => {
            show_selection(state, &ids, &groups)?;
            Ok(InternalRespondResult {
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
            let res = Ok(to_internal(state.toggle_loop(&ids, &groups, duration)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Unloop { ids, groups } => {
            let res = Ok(to_internal(state.unloop(&ids, &groups)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::SetStart {
            ids,
            groups,
            pos: duration,
        } => {
            let res = Ok(to_internal(state.set_start(&ids, &groups, duration)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::SetEnd {
            ids,
            groups,
            pos: duration,
        } => {
            let res = Ok(to_internal(state.set_end(&ids, &groups, duration)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Delay {
            ids,
            groups,
            duration,
        } => {
            let res = Ok(to_internal(state.delay(&ids, &groups, duration)?));
            show_selection(state, &ids, &groups)?;
            res
        }
        Commands::Group {
            group: group_name,
            ids,
        } => Ok(to_internal(state.group(group_name, &ids)?)),
        Commands::Ungroup { group, ids } => Ok(to_internal(state.ungroup(group, &ids)?)),
        Commands::Save { path } => Ok(to_internal(state.save(&path)?)),
        Commands::Load { path } => load_combine_or_overwrite(state, path, has_been_saved, rl),
        Commands::Exit => Ok(InternalRespondResult {
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
) -> Result<InternalRespondResult, Error> {
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
            let new = AppState::load(&path)?;
            state.players = new.players;
            state.top_group = new.top_group;
            state.groups = new.groups;
            Ok(InternalRespondResult {
                saved: true,
                mutated: true,
                quit: false,
            })
        } else {
            Ok(InternalRespondResult {
                saved: false,
                mutated: false,
                quit: false,
            })
        };
    }

    let mut new = AppState::load(&path)?;

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
        new.players,
        state.players,
        vec![("Overwrite", "o"), ("Rename", "r"), ("Skip", "s")]
    );

    let (group_renames, groups_to_skip) = get_changes_helper!(
        "group",
        new.groups,
        state.groups,
        vec![("Merge", "m"), ("Rename", "r"), ("Skip", "s")]
    );

    for (name, new_name) in player_renames {
        if let Some(mut player) = new.players.remove(&name) {
            player.name = new_name.clone();
            new.players.insert(new_name.clone(), player);
        }

        if let Some((index, _)) = new.top_group.shift_remove_full(&name) {
            new.top_group.shift_insert(index, new_name.clone());
        }

        for group in new.groups.values_mut() {
            let res = group.shift_remove_full(&name);
            if let Some((index, _)) = res {
                group.shift_insert(index, new_name.clone());
            }
        }
    }

    for skip in players_to_skip {
        new.players.remove(&skip);
        new.top_group.shift_remove(&skip);
        for group in new.groups.values_mut() {
            group.shift_remove(&skip);
        }
    }

    for skip in groups_to_skip {
        new.groups.shift_remove(&skip);
    }

    for (name, new_name) in group_renames {
        if let Some((index, _, group)) = new.groups.shift_remove_full(&name) {
            new.groups.shift_insert(index, new_name.clone(), group);
        }
    }

    state.players.extend(new.players);
    state.top_group.extend(new.top_group);
    for (name, new_group) in new.groups {
        if let Some(group) = state.groups.get_mut(&name) {
            group.extend(new_group);
        } else {
            state.groups.insert(name, new_group);
        }
    }

    Ok(InternalRespondResult {
        saved: false,
        mutated: true,
        quit: false,
    })
}

fn to_internal(base: RespondResult) -> InternalRespondResult {
    InternalRespondResult {
        saved: base.saved,
        mutated: base.mutated,
        quit: false,
    }
}
