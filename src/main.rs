use anyhow::Error;
use clap::Parser;
use const_format::formatcp;
use operations::{
    add, delay, exit, load, pause, play, remove, save, set_end, set_start, set_volume, show, stop,
    toggle_loop, unloop, RespondResult,
};
use player::Player;
use std::io::Write;
use std::{path::PathBuf, time::Duration};

mod operations;
mod player;

//TODO: propper error messages
//TODO: reset command
//FUTURE: implement grouping
//FAR FUTURE: make a nice GUI
//VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

const ADD_USAGE: &str = "add -p <PATH> -n <NAME>";
const REMOVE_USAGE: &str = "remove [IDs]";
const SHOW_USAGE: &str = "show [IDs]";
const PLAY_USAGE: &str = "play [IDs]";
const STOP_USAGE: &str = "stop [IDs]";
const PAUSE_USAGE: &str = "pause [IDs]";
const VOLUME_USAGE: &str = "volume [IDs] -v <VOLUME>";
const LOOP_USAGE: &str = "loop [IDs] [-d <DURATION>]";
const UNLOOP_USAGE: &str = "unloop [IDs]";
const SET_START_USAGE: &str = "set-start [IDs] -p <POS>";
const SET_END_USAGE: &str = "set-end [IDs] [-p <POS>]";
const DELAY_USAGE: &str = "delay [IDs] -d <DURATION>";

const NO_ID_ADDENDUM: &str = "When called without ID, this will select the last added sound.";

const ABOUT_ADD: &str = "Will add a sound to the soundscape.";
const ABOUT_ADD_LONG: &str = "Will add a sound to the soundscape. Added sounds will not start playing until you call play [ID].";
const ABOUT_REMOVE: &str = "Will remove a sound from the soundscape.";
const ABOUT_VOLUME: &str = "Set volume as a percentage. Can be higher than 100%";
const ABOUT_SHOW: &str = "Will show the status of a sound.";
const ABOUT_LOOP: &str = "Will loop at the end of the sound or the DURATION, if supplied.";
const ABOUT_LOOP_LONG: &str = "Will loop at the end of the sound or the DURATION, if supplied. DURATION can be longer than the sound length.";
const ABOUT_UNLOOP: &str = "Turns of looping for this sound.";
const ABOUT_SET_START: &str = "Clips the start of a sound by selecting the starting position.";
const ABOUT_SET_END: &str =
    "Clips the end of a sound by selecting the ending position. Reset by omitting POS.";
const ABOUT_DELAY: &str =
    "Delays playing the sound after the play command. Useful in combination with  play-all.";
const ABOUT_HELP: &str = "Shows this help message.";
const ABOUT_EXIT: &str = "Exits the program.";

const HELP_MESSAGE: &str = formatcp!(
    "\
{{name}}: {{about}}

Usage:
\t{ADD_USAGE}\t\t{ABOUT_ADD}
\t{REMOVE_USAGE}\t\t\t{ABOUT_REMOVE}
\t{SHOW_USAGE}\t\t\t{ABOUT_SHOW}
\t{PLAY_USAGE}
\t{STOP_USAGE}
\t{PAUSE_USAGE}
\t{VOLUME_USAGE}\t{ABOUT_VOLUME}
\t{LOOP_USAGE}\t{ABOUT_LOOP}
\t{UNLOOP_USAGE}\t\t\t{ABOUT_UNLOOP}
\t{SET_START_USAGE}\t{ABOUT_SET_START}
\t{SET_END_USAGE}\t{ABOUT_SET_END}
\t{DELAY_USAGE}\t{ABOUT_DELAY}
\thelp\t\t\t\t{ABOUT_HELP}
\texit\t\t\t\t{ABOUT_EXIT}

Note that:
\t- [] indicates an optional value.
\t- Most commands will select the last added sound if ID is not supplied.
\t- ID can be a name, 'all', or an index. For instance: 'play horn', 'play all' or 'play 0'\
"
);

const COMMAND_HELP: &str = "\
usage: {usage}

{about}\
";

macro_rules! build {
    ($($(#$macro:tt)? $ident:ident $({$($body:tt)*})?),*) => {
        #[derive(Debug, Parser)]
        #[command(no_binary_name = true, help_template = HELP_MESSAGE, about = "A simple audio looping application for the creation of soundscapes.")]
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
    #[command(override_usage=REMOVE_USAGE, about=format!("{ABOUT_REMOVE} {NO_ID_ADDENDUM}"))]
    Remove {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=PLAY_USAGE, about=NO_ID_ADDENDUM)]
    Play {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=STOP_USAGE, about=NO_ID_ADDENDUM)]
    Stop {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=PAUSE_USAGE, about=NO_ID_ADDENDUM)]
    Pause {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=VOLUME_USAGE, about=format!("{ABOUT_VOLUME} {NO_ID_ADDENDUM}"))]
    Volume {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
        #[arg(long, short)]
        volume: u32
    },
    #[command(override_usage=SHOW_USAGE, about=format!("{ABOUT_SHOW} {NO_ID_ADDENDUM}"))]
    Show {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=LOOP_USAGE, about=format!("{ABOUT_LOOP_LONG} {NO_ID_ADDENDUM}"))]
    Loop {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Option<Duration>,
    },
    #[command(override_usage=UNLOOP_USAGE, about=format!("{ABOUT_UNLOOP} {NO_ID_ADDENDUM}"))]
    Unloop {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
    },
    #[command(override_usage=SET_START_USAGE, about=format!("{ABOUT_SET_START} {NO_ID_ADDENDUM}"))]
    SetStart {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
        #[arg(long, short, value_parser = parse_duration)]
        pos: Duration,
    },
    #[command(override_usage=SET_END_USAGE, about=format!("{ABOUT_SET_END} {NO_ID_ADDENDUM}"))]
    SetEnd {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
        #[arg(long, short, value_parser = parse_duration)]
        pos: Option<Duration>,
    },
    #[command(override_usage=DELAY_USAGE, about=format!("{ABOUT_DELAY} {NO_ID_ADDENDUM}"))]
    Delay {
        #[arg(value_parser = parse_id)]
        ids: Vec<IdOrName>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Duration,
    },
    Save {
        #[arg(long, short)]
        path: PathBuf,
    },
    Load {
        #[arg(long, short)]
        path: PathBuf,
    },
    #[command(about=ABOUT_EXIT)]
    Exit
}

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
enum IdOrName {
    Id(usize),
    All,
    Name(String),
}

fn parse_id(id: &str) -> Result<IdOrName, Error> {
    let int_result = id.parse::<usize>();
    int_result.map_or_else(
        |_| {
            if &id.to_lowercase() == "all" {
                Ok(IdOrName::All)
            } else {
                Ok(IdOrName::Name(id.to_string()))
            }
        },
        |res| Ok(IdOrName::Id(res)),
    )
}

fn parse_duration(dur: &str) -> Result<Duration, Error> {
    Ok(duration_str::parse(dur)?)
}

fn main() -> Result<(), String> {
    let mut players = Vec::new();
    let mut has_been_saved = true;
    loop {
        let line = readline("$ ").map_err(|e| e.to_string())?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(&mut players, line, has_been_saved) {
            Ok(RespondResult {
                saved,
                mutated,
                quit,
            }) => {
                has_been_saved = (has_been_saved || saved) && !mutated;
                if quit {
                    break Ok(());
                }
            }
            Err(err) => {
                println!("{err}");
            }
        }
    }
}

fn respond(
    players: &mut Vec<Player>,
    line: &str,
    has_been_saved: bool,
) -> Result<RespondResult, Error> {
    let args = shlex::split(line).ok_or_else(|| {
        Error::msg("error: cannot parse input. Perhaps you have erronous quotation(\"\")?")
    })?;
    let matches = Commands::try_parse_from(args)?;
    match matches {
        Commands::Add { path, name } => add(players, path, name),
        Commands::Remove { ids } => remove(players, ids),
        Commands::Play { ids } => play(players, ids),
        Commands::Stop { ids } => stop(players, ids),
        Commands::Pause { ids } => pause(players, ids),
        Commands::Volume { ids, volume } => set_volume(players, ids, volume),
        Commands::Show { ids } => show(players, ids),
        Commands::Loop { ids, duration } => toggle_loop(players, ids, duration),
        Commands::Unloop { ids } => unloop(players, ids),
        Commands::SetStart { ids, pos: duration } => set_start(players, ids, duration),
        Commands::SetEnd { ids, pos: duration } => set_end(players, ids, duration),
        Commands::Delay { ids, duration } => delay(players, ids, duration),
        Commands::Save { path } => save(players, &path),
        Commands::Load { path } => load(players, &path, has_been_saved),
        Commands::Exit => exit(has_been_saved),
    }
}

pub fn readline(prompt: &str) -> Result<String, Error> {
    write!(std::io::stdout(), "{prompt}").map_err(|e| Error::msg(e.to_string()))?;
    std::io::stdout()
        .flush()
        .map_err(|e| Error::msg(e.to_string()))?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| Error::msg(e.to_string()))?;
    Ok(buffer)
}

fn get_confirmation(prompt: &str) -> Result<bool, Error> {
    Ok(readline(format!("{prompt} Y/N: ").as_str())
        .map_err(Error::msg)?
        .trim()
        .to_lowercase()
        == "y")
}
