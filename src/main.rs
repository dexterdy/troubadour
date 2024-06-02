use anyhow::Error;
use clap::Parser;
use const_format::formatcp;
use indexmap::{IndexMap, IndexSet};
use operations::{
    add, delay, exit, group, load, pause, play, remove, save, set_end, set_start, set_volume, show,
    stop, toggle_loop, ungroup, unloop, RespondResult,
};
use player::Player;
use rustyline::error::ReadlineError;
use rustyline::history::FileHistory;
use rustyline::{DefaultEditor, Editor};
use std::cell::RefCell;
use std::collections::HashMap;
use std::{path::PathBuf, time::Duration};

mod operations;
mod player;

//TODO: Implement a sound length feature, based on amount samples
//TODO: add fades toggle
//TODO: make a nice GUI
//VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

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

// FIXME: this only works if the app stays single threaded. Also, when I write the GUI version, this should probably be refactored.
// additionally, It prevents any debugger from working;
thread_local! {static READLINE: RefCell<Editor<(), FileHistory>> = RefCell::new(DefaultEditor::new().expect("error: could not get access to the stdin."))}

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

    let mut state = AppState {
        players: HashMap::new(),
        top_group: IndexSet::new(),
        groups: IndexMap::new(),
    };

    let mut has_been_saved = true;

    loop {
        let mut should_quit = false;

        let response = readline("$ ").and_then(|line| {
            let line = line.trim();
            respond(&mut state, &line, has_been_saved)
        });

        match response {
            Ok(RespondResult {
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
                || get_confirmation("Are you sure you want to exit without saving?")
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

fn respond(state: &mut AppState, line: &str, has_been_saved: bool) -> Result<RespondResult, Error> {
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
        Commands::Add { path, name } => add(state, path, name),
        Commands::Remove { ids } => remove(state, ids),
        Commands::Play { ids, groups } => play(state, ids, groups),
        Commands::Stop { ids, groups } => stop(state, ids, groups),
        Commands::Pause { ids, groups } => pause(state, ids, groups),
        Commands::Volume {
            ids,
            groups,
            volume,
        } => set_volume(state, ids, groups, volume),
        Commands::Show { ids, groups } => show(state, ids, groups),
        Commands::Loop {
            ids,
            groups,
            duration,
        } => toggle_loop(state, ids, groups, duration),
        Commands::Unloop { ids, groups } => unloop(state, ids, groups),
        Commands::SetStart {
            ids,
            groups,
            pos: duration,
        } => set_start(state, ids, groups, duration),
        Commands::SetEnd {
            ids,
            groups,
            pos: duration,
        } => set_end(state, ids, groups, duration),
        Commands::Delay {
            ids,
            groups,
            duration,
        } => delay(state, ids, groups, duration),
        Commands::Group {
            group: group_name,
            ids,
        } => group(state, group_name, ids),
        Commands::Ungroup { group, ids } => ungroup(state, group, ids),
        Commands::Save { path } => save(state, &path),
        Commands::Load { path } => load(state, &path, has_been_saved),
        Commands::Exit => exit(),
    }
}

pub fn readline(prompt: &str) -> Result<String, Error> {
    READLINE.with_borrow_mut(|rl| {
        let line = rl.readline(prompt);
        match line {
            Ok(line) => {
                rl.add_history_entry(line.as_str()).unwrap_or_default();
                Ok(line)
            }
            Err(ReadlineError::Eof) => Err(Error::msg("error: unexpected EOF.")),
            Err(ReadlineError::WindowResized) => readline(prompt),
            Err(ReadlineError::Interrupted) => Ok(line?),
            _ => Err(Error::msg("error: could not read from stdin")),
        }
    })
}

fn get_confirmation(prompt: &str) -> Result<bool, Error> {
    let mut result = None;

    while result.is_none() {
        let response = readline(format!("{prompt} Y/N: ").as_str())
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

fn get_option(prompt: &str, valid_options: Vec<&str>) -> Result<String, Error> {
    let mut result = None;

    while result.is_none() {
        let response = readline(format!("{prompt}: ").as_str())
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
