use anyhow::Error;
use clap::Parser;
use const_format::formatcp;
use std::{path::PathBuf, time::Duration};

const ADD_USAGE: &str = "add -p <PATH> -n <NAME>";
const REMOVE_USAGE: &str = "remove [IDs]";
const COPY_USAGE: &str = "copy [IDs] [-g <GROUPS>]";
const SHOW_USAGE: &str = "show [IDs] [-g <GROUPS>]";
const PLAY_USAGE: &str = "play [IDs] [-g <GROUPS>]";
const STOP_USAGE: &str = "stop [IDs] [-g <GROUPS>]";
const PAUSE_USAGE: &str = "pause [IDs] [-g <GROUPS>]";
const VOLUME_USAGE: &str = "volume [IDs] [-g <GROUPS>] -v <VOLUME>";
const LOOP_USAGE: &str = "loop [IDs] [-g <GROUPS>] [-d <DURATION>]";
const UNLOOP_USAGE: &str = "unloop [IDs] [-g <GROUPS>]";
const CUT_START_USAGE: &str = "cut-start [IDs] [-g <GROUPS>] -d <DURATION>";
const CUT_END_USAGE: &str = "cut-end [IDs] [-g <GROUPS>] -d <DURATION>";
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
const ABOUT_COPY: &str = "Copies sounds and their settings";
const ABOUT_VOLUME: &str = "Sets the volume as a percentage. Can be higher than 100%";
const ABOUT_SHOW: &str = "Shows the status and configuration of sounds.";
const ABOUT_PLAY: &str = "Plays sounds.";
const ABOUT_STOP: &str = "Stops sounds and resets the play heads to the start of each sound.";
const ABOUT_PAUSE: &str = "Pauses sounds.";
const ABOUT_LOOP: &str = "Loops sounds at the end of their play length or DURATION, if supplied.";
const ABOUT_LOOP_LONG: &str = "Loops sounds the end of their play length or the DURATION, if supplied. DURATION can be longer than the sounds lengths.";
const ABOUT_UNLOOP: &str = "Turns of looping for these sounds.";
const ABOUT_CUT_START: &str = "Clips the start of sounds.";
const ABOUT_CUT_END: &str = "Clips the end of sounds.";
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

\t{COPY_USAGE}\n\t\t{ABOUT_COPY}

\t{SHOW_USAGE}\n\t\t{ABOUT_SHOW}

\t{PLAY_USAGE}\n\t\t{ABOUT_PLAY}

\t{STOP_USAGE}\n\t\t{ABOUT_STOP}

\t{PAUSE_USAGE}\n\t\t{ABOUT_PAUSE}

\t{VOLUME_USAGE}\n\t\t{ABOUT_VOLUME}

\t{LOOP_USAGE}\n\t\t{ABOUT_LOOP}

\t{UNLOOP_USAGE}\n\t\t{ABOUT_UNLOOP}

\t{CUT_START_USAGE}\n\t\t{ABOUT_CUT_START}

\t{CUT_END_USAGE}\n\t\t{ABOUT_CUT_END}

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
        pub enum Commands {$(
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
        ids: Vec<String>
    },
    #[command(override_usage=COPY_USAGE, about=format!("{ABOUT_COPY} {NO_ID_ADDENDUM}"))]
    Copy {
        ids: Vec<String>,
        #[arg(long, short)]
        groups: Vec<String>
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
    #[command(override_usage=CUT_START_USAGE, about=format!("{ABOUT_CUT_START} {NO_ID_ADDENDUM}"))]
    CutStart {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Duration,
        #[arg(long, short)]
        groups: Vec<String>
    },
    #[command(override_usage=CUT_END_USAGE, about=format!("{ABOUT_CUT_END} {NO_ID_ADDENDUM}"))]
    CutEnd {
        ids: Vec<String>,
        #[arg(long, short, value_parser = parse_duration)]
        duration: Duration,
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
        ids: Vec<String>
    },
    #[command(override_usage=UNGROUP_USAGE, about=ABOUT_UNGROUP)]
    Ungroup {
        #[arg(long, short)]
        group: String,
        ids: Vec<String>
    },
    #[command(override_usage=SAVE_USAGE, about=ABOUT_SAVE)]
    Save {
        #[arg(long, short)]
        path: PathBuf
    },
    #[command(override_usage=LOAD_USAGE, about=ABOUT_LOAD)]
    Load {
        #[arg(long, short)]
        path: PathBuf
    },
    #[command(about=ABOUT_EXIT)]
    Exit
}

fn parse_duration(dur: &str) -> Result<Duration, Error> {
    Ok(duration_str::parse(dur).map_err(|e| Error::msg(e))?)
}
