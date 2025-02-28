use anyhow::Error;
use clap::Parser;
use const_format::formatcp;
use duration_human::DurationHuman;
use fomat_macros::fomat;
use indexmap::{IndexMap, IndexSet};
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};
use std::{path::PathBuf, time::Duration};
use troubadour_lib::player::Player;
use troubadour_lib::{AppState, RespondResult};

// TODO: Implement a sound length feature, based on amount samples
// TODO: add fades toggle
// TODO: make a nice GUI
// TODO: write a bunch of tests
// VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

const ADD_USAGE: &str = "add -p <PATH> -n <NAME>";
const REMOVE_USAGE: &str = "remove [IDs]";
const SHOW_USAGE: &str = "show [IDs] [-g <GROUPS>]";
const PLAY_USAGE: &str = "play [IDs] [-g <GROUPS>]";
const STOP_USAGE: &str = "stop [IDs] [-g <GROUPS>]";
const PAUSE_USAGE: &str = "pause [IDs] [-g <GROUPS>]";
const VOLUME_USAGE: &str = "volume [IDs] [-g <GROUPS>] -v <VOLUME>";
const LOOP_USAGE: &str = "loop [IDs] [-g <GROUPS>] [-d <DURATION>]";
const UNLOOP_USAGE: &str = "unloop [IDs] [-g <GROUPS>]";
const SET_START_USAGE: &str = "set-start [IDs] [-g <GROUPS>] -p <POS>";
const SET_END_USAGE: &str = "set-end [IDs] [-g <GROUPS>] [-p <POS>]";
const DELAY_USAGE: &str = "delay [IDs] [-g <GROUPS>] -d <DURATION>";
const GROUP_USAGE: &str = "group [IDs] -g <GROUP>";
const UNGROUP_USAGE: &str = "ungroup [IDs] -g <GROUP>";
const SAVE_USAGE: &str = "save -p <PATH>";
const LOAD_USAGE: &str = "load -p <PATH>";

const NO_ID_ADDENDUM: &str = "When called without ID, this will select the last added sound.";

const ABOUT_ADD: &str = "Adds a sound to the soundscape.";
const ABOUT_ADD_LONG: &str =
    "Adds a sound to the soundscape. Added sounds will not start playing until you call play.";
const ABOUT_REMOVE: &str = "Removes sounds from the soundscape.";
const ABOUT_VOLUME: &str = "Sets the volume as a percentage. Can be higher than 100%";
const ABOUT_SHOW: &str = "Shows the status and configuration of sounds.";
const ABOUT_PLAY: &str = "Plays sounds.";
const ABOUT_STOP: &str = "Stops sounds and resets the play heads to the start of each sound.";
const ABOUT_PAUSE: &str = "Pauses sounds.";
const ABOUT_LOOP: &str = "Loops sounds at the end of their play length or DURATION, if supplied.";
const ABOUT_LOOP_LONG: &str = "Loops sounds the end of their play length or the DURATION, if supplied. DURATION can be longer than the sounds lengths.";
const ABOUT_UNLOOP: &str = "Turns of looping for these sounds.";
const ABOUT_SET_START: &str = "Clips the start of sounds by selecting the starting position.";
const ABOUT_SET_END: &str =
    "Clips the end of sounds by selecting the ending position. Reset by omitting POS.";
const ABOUT_DELAY: &str =
    "Delays playing the sound after the play command. Useful when you play multiple sounds at once.";
const ABOUT_GROUP: &str =
    "Adds sounds to a group. If the group doesn't exists yet, a new one will be made.";
const ABOUT_UNGROUP: &str =
    "Removes sounds from a group. If the group is empty after this operation, it will be removed.";
const ABOUT_SAVE: &str = "Saves the current configuration to a file.";
const ABOUT_LOAD: &str =
    "Loads a saved configuration. You can choose to replace or add to current configuration.";
const ABOUT_HELP: &str = "Shows this help message.";
const ABOUT_EXIT: &str = "Exits the program.";

const USAGE: &str = formatcp!(
    "
\t{ADD_USAGE}\n\t\t{ABOUT_ADD}

\t{REMOVE_USAGE}\n\t\t{ABOUT_REMOVE}

\t{SHOW_USAGE}\n\t\t{ABOUT_SHOW}

\t{PLAY_USAGE}\n\t\t{ABOUT_PLAY}

\t{STOP_USAGE}\n\t\t{ABOUT_STOP}

\t{PAUSE_USAGE}\n\t\t{ABOUT_PAUSE}

\t{VOLUME_USAGE}\n\t\t{ABOUT_VOLUME}

\t{LOOP_USAGE}\n\t\t{ABOUT_LOOP}

\t{UNLOOP_USAGE}\n\t\t{ABOUT_UNLOOP}

\t{SET_START_USAGE}\n\t\t{ABOUT_SET_START}

\t{SET_END_USAGE}\n\t\t{ABOUT_SET_END}

\t{DELAY_USAGE}\n\t\t{ABOUT_DELAY}

\t{GROUP_USAGE}\n\t\t{ABOUT_GROUP}

\t{UNGROUP_USAGE}\n\t\t{ABOUT_UNGROUP}

\t{SAVE_USAGE}\n\t\t{ABOUT_SAVE}

\t{LOAD_USAGE}\n\t\t{ABOUT_LOAD}

\thelp\n\t\t{ABOUT_HELP}

\texit\n\t\t{ABOUT_EXIT}

Note that:
\t- [..] indicates an optional value.
\t- Most commands will select the last added sound if ID is not supplied.
\t- ID can be a name or 'all'. For instance: 'play horn' or 'play all'\
"
);

const HELP_MESSAGE: &str = "\
{name}: {about}

Usage: {usage}\
";

const COMMAND_HELP: &str = "\
usage: {usage}

{about}\
";

macro_rules! build {
    ($($(#$macro:tt)? $ident:ident $({$($body:tt)*})?),*) => {
        #[derive(Debug, Parser)]
        #[command(no_binary_name = true, help_template = HELP_MESSAGE, override_usage = USAGE, about = "A simple audio looping application for the creation of soundscapes.")]
        enum Commands {$(
            #[command(no_binary_name = true, allow_missing_positional = true, help_template = COMMAND_HELP)]
            $(#$macro)?
            $ident $({$($body)*})?,
        )*}
    };
}

build! {
    #[command(override_usage=ADD_USAGE, about=ABOUT_ADD_LONG)]
    Add {
        #[arg(long, short)]
        path: PathBuf,
        #[arg(long, short)]
        name: String
    },
    #[command(override_usage=REMOVE_USAGE, about=ABOUT_REMOVE)]
    Remove {
        ids: Vec<String>,
    },
    #[command(override_usage=PLAY_USAGE, about=format!("{ABOUT_PLAY} {NO_ID_ADDENDUM}"))]
    Play {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=STOP_USAGE, about=format!("{ABOUT_STOP} {NO_ID_ADDENDUM}"))]
    Stop {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=PAUSE_USAGE, about=format!("{ABOUT_PAUSE} {NO_ID_ADDENDUM}"))]
    Pause {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=VOLUME_USAGE, about=format!("{ABOUT_VOLUME} {NO_ID_ADDENDUM}"))]
    Volume {
        ids: Vec<String>,
        #[arg(long, short)]
        volume: u32,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=SHOW_USAGE, about=format!("{ABOUT_SHOW} {NO_ID_ADDENDUM}"))]
    Show {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=LOOP_USAGE, about=format!("{ABOUT_LOOP_LONG} {NO_ID_ADDENDUM}"))]
    Loop {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Option<Duration>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=UNLOOP_USAGE, about=format!("{ABOUT_UNLOOP} {NO_ID_ADDENDUM}"))]
    Unloop {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=SET_START_USAGE, about=format!("{ABOUT_SET_START} {NO_ID_ADDENDUM}"))]
    SetStart {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        pos: Duration,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=SET_END_USAGE, about=format!("{ABOUT_SET_END} {NO_ID_ADDENDUM}"))]
    SetEnd {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        pos: Option<Duration>,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=DELAY_USAGE, about=format!("{ABOUT_DELAY} {NO_ID_ADDENDUM}"))]
    Delay {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Duration,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=GROUP_USAGE, about=ABOUT_GROUP)]
    Group {
        #[arg(long, short)]
        group: String,
        ids: Vec<String>,
    },
    #[command(override_usage=UNGROUP_USAGE, about=ABOUT_UNGROUP)]
    Ungroup {
        #[arg(long, short)]
        group: String,
        ids: Vec<String>,
    },
    #[command(override_usage=SAVE_USAGE, about=ABOUT_SAVE)]
    Save {
        #[arg(long, short)]
        path: PathBuf,
    },
    #[command(override_usage=LOAD_USAGE, about=ABOUT_LOAD)]
    Load {
        #[arg(long, short)]
        path: PathBuf,
    },
    #[command(about=ABOUT_EXIT)]
    Exit
}

fn parse_duration(dur: &str) -> Result<Duration, Error> {
    Ok(duration_str::parse(dur)?)
}

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

    let mut has_been_saved = true;

    loop {
        let mut should_quit = false;

        let mut state = AppState::new();

        let response = readline("$ ", &mut rl).and_then(|line| {
            let line = line.trim();
            respond(&mut state, &line, has_been_saved, &mut rl)
        });

        match response {
            Ok(InternalRespondResult {
                saved,
                mutated,
                quit,
            }) => {
                has_been_saved = (has_been_saved || saved) && !mutated;
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
    let option = get_option(
        "Do you want to combine workspaces? Combine(C)/Overwrite(O)",
        vec!["c", "o"],
        rl,
    )?;

    if option == "O" {
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
                saved: false,
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
    let mut renames = vec![];
    let mut to_skip = vec![];

    for name in new.players.keys() {
        if state.players.contains_key(name) {
            let option = get_option(
                &format!(
                    "A player by the name of {name} already exists. Overwrite(O)/Skip(S)/Rename(R)"
                ),
                vec!["o", "s", "r"],
                rl,
            )?;
            match option.as_str() {
                "o" => (),
                "r" => {
                    let new_name: String = readline("What should the new name be?", rl)?;
                    renames.push((name.clone(), new_name));
                }
                _ => {
                    to_skip.push(name.clone());
                }
            }
        }
    }

    for (name, new_name) in renames {
        let mut player = new.players.remove(&name).unwrap();
        player.name = new_name.clone();
        new.players.insert(new_name.clone(), player);
        let (index, _) = new.top_group.shift_remove_full(&name).unwrap();
        new.top_group.shift_insert(index, new_name.clone());
        for group in new.groups.values_mut() {
            let res = group.shift_remove_full(&name);
            if let Some((index, _)) = res {
                group.shift_insert(index, new_name.clone());
            }
        }
    }

    for skip in to_skip {
        new.players.remove(&skip);
        new.top_group.shift_remove(&skip);
        for group in new.groups.values_mut() {
            group.shift_remove(&skip);
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

pub fn readline(prompt: &str, rl: &mut Editor<(), FileHistory>) -> Result<String, Error> {
    let line = rl.readline(&prompt);
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

fn get_confirmation(prompt: &str, rl: &mut Editor<(), FileHistory>) -> Result<bool, Error> {
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

fn get_option(
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

fn duration_to_string(dur: Duration, no_smaller_than_secs: bool) -> String {
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

fn player_to_string(player: &Player) -> String {
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

fn show_selection(
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
    if ids.len() == 0 && group_ids.len() == 0 && state.top_group.len() > 0 {
        print_player(state.top_group.last().unwrap())?;
    }
    Ok(())
}
