use anyhow::Error;
use clap::Parser;
use const_format::formatcp;
use player::{Player, SerialisablePlayer};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::Write;
use std::{path::PathBuf, time::Duration};

//TODO: propper error messages
//TODO: reset command
//FUTURE: implement grouping
//FAR FUTURE: make a nice GUI
//VERY FAR FUTURE: add a special mapping feature (dungeon vtt-esque)

const ADD_USAGE: &'static str = "add -p <PATH> -n <NAME>";
const REMOVE_USAGE: &'static str = "remove [IDs]";
const SHOW_USAGE: &'static str = "show [IDs]";
const PLAY_USAGE: &'static str = "play [IDs]";
const STOP_USAGE: &'static str = "stop [IDs]";
const PAUSE_USAGE: &'static str = "pause [IDs]";
const VOLUME_USAGE: &'static str = "volume [IDs] -v <VOLUME>";
const LOOP_USAGE: &'static str = "loop [IDs] [-d <DURATION>]";
const UNLOOP_USAGE: &'static str = "unloop [IDs]";
const SET_START_USAGE: &'static str = "set-start [IDs] -p <POS>";
const SET_END_USAGE: &'static str = "set-end [IDs] [-p <POS>]";
const DELAY_USAGE: &'static str = "delay [IDs] -d <DURATION>";

const NO_ID_ADDENDUM: &'static str =
    "When called without ID, this will select the last added sound.";

const ABOUT_ADD: &'static str = "Will add a sound to the soundscape.";
const ABOUT_ADD_LONG: &'static str = "Will add a sound to the soundscape. Added sounds will not start playing until you call play [ID].";
const ABOUT_REMOVE: &'static str = "Will remove a sound from the soundscape.";
const ABOUT_VOLUME: &'static str = "Set volume as a percentage. Can be higher than 100%";
const ABOUT_SHOW: &'static str = "Will show the status of a sound.";
const ABOUT_LOOP: &'static str = "Will loop at the end of the sound or the DURATION, if supplied.";
const ABOUT_LOOP_LONG: &'static str = "Will loop at the end of the sound or the DURATION, if supplied. DURATION can be longer than the sound length.";
const ABOUT_UNLOOP: &'static str = "Turns of looping for this sound.";
const ABOUT_SET_START: &'static str =
    "Clips the start of a sound by selecting the starting position.";
const ABOUT_SET_END: &'static str =
    "Clips the end of a sound by selecting the ending position. Reset by omitting POS.";
const ABOUT_DELAY: &'static str =
    "Delays playing the sound after the play command. Useful in combination with  play-all.";
const ABOUT_HELP: &'static str = "Shows this help message.";
const ABOUT_EXIT: &'static str = "Exits the program.";

const HELP_MESSAGE: &'static str = formatcp!(
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

const COMMAND_HELP: &'static str = "\
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
    match int_result {
        Ok(res) => Ok(IdOrName::Id(res)),
        Err(_) => {
            if &id.to_lowercase() == "all" {
                Ok(IdOrName::All)
            } else {
                Ok(IdOrName::Name(id.to_string()))
            }
        }
    }
}

fn parse_duration(dur: &str) -> Result<Duration, Error> {
    Ok(duration_str::parse(dur)?)
}

mod player;

fn main() -> Result<(), String> {
    let mut players = Vec::new();
    let mut has_been_saved = true;
    loop {
        let line = readline("$ ")?;
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

struct RespondResult {
    mutated: bool,
    saved: bool,
    quit: bool,
}

fn respond(
    players: &mut Vec<Player>,
    line: &str,
    has_been_saved: bool,
) -> Result<RespondResult, Error> {
    let args = shlex::split(line).ok_or(Error::msg(
        "error: cannot parse input. Perhaps you have erronous quotation(\"\")?",
    ))?;
    let matches = Commands::try_parse_from(args)?;
    let mut mutated = false;
    let mut saved = false;
    match matches {
        Commands::Add { path, name } => {
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
            if players.into_iter().filter(|p| p.name == name).count() > 0 {
                return Err(Error::msg(format!(
                    "error: you cannot use the name '{name}', because it is already used."
                )));
            }
            let new_player = Player::new(path, name)?;
            println!("{}", new_player.to_string());
            players.push(new_player);
            mutated = true;
        }
        Commands::Remove { ids } => {
            // unsafe to have a mut and not mut ref at the same time
            // make sure to print before remove
            // this is an ugly hack
            let mut selected_players =
                select_players(unsafe { &*(players as *const Vec<Player>) }, ids)?
                    .iter()
                    .map(|p| p.name.as_str())
                    .enumerate()
                    .collect::<Vec<(usize, &str)>>();
            if get_confirmation("Are you sure you want to remove these sounds?")? {
                println!(
                    "Removed {}",
                    selected_players
                        .iter()
                        .map(|(_, p)| *p)
                        .collect::<Vec<&str>>()
                        .join(", ")
                );
                selected_players.sort_by_key(|p| p.0);
                selected_players.reverse();
                for (pos, _) in selected_players.clone() {
                    players.remove(pos);
                }
                mutated = true;
            }
        }
        Commands::Play { ids } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(&mut player, play()?);
            show_selection!(player);
        }
        Commands::Stop { ids } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(&mut player, stop());
            show_selection!(player);
        }
        Commands::Pause { ids } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(&mut player, pause());
            show_selection!(player);
        }
        Commands::Volume { ids, volume } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(&mut player, volume(volume));
            show_selection!(player);
            mutated = true;
        }
        Commands::Show { ids } => {
            let player = select_players(players, ids)?;
            show_selection!(player);
        }
        Commands::Loop { ids, duration } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(
                &mut player,
                toggle_loop(true),
                loop_length(duration),
                apply_settings_in_place(false)?
            );
            show_selection!(player);
            mutated = true;
        }
        Commands::Unloop { ids } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(
                &mut player,
                toggle_loop(false),
                apply_settings_in_place(false)?
            );
            show_selection!(player);
            mutated = true;
        }
        Commands::SetStart { ids, pos: duration } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(
                &mut player,
                skip_duration(duration),
                apply_settings_in_place(false)?
            );
            show_selection!(player);
            mutated = true
        }
        Commands::SetEnd { ids, pos: duration } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(
                &mut player,
                take_duration(duration),
                apply_settings_in_place(false)?
            );
            show_selection!(player);
            mutated = true
        }
        Commands::Delay { ids, duration } => {
            let mut player = select_players_mut(players, ids)?;
            apply_vec!(
                &mut player,
                set_delay(duration),
                apply_settings_in_place(false)?
            );
            show_selection!(player);
            mutated = true
        }
        Commands::Save { path } => {
            let serialisable: Vec<SerialisablePlayer> =
                players.into_iter().map(|p| p.to_serializable()).collect();
            let json = serde_json::to_string(&serialisable)?;
            fs::write(path, json)?;
            saved = true;
        }
        Commands::Load { path } => {
            let add_to_soundscape = players.len() == 0
                || get_confirmation("Do you want to add this to you current soundscape?")?;
            let perform_action = add_to_soundscape
                || has_been_saved
                || get_confirmation("Are you sure you want to exit without saving?")?;
            if perform_action {
                let json: Vec<SerialisablePlayer> = serde_json::from_reader(File::open(path)?)?;
                let new_players = json
                    .into_iter()
                    .map(|p| Ok(Player::from_serializable(&p)?))
                    .collect::<Result<Vec<Player>, Error>>()?;
                if !add_to_soundscape {
                    players.clear();
                }
                players.extend(new_players);
                show_selection!(players)
            }
        }
        Commands::Exit => {
            let perform_action = has_been_saved
                || get_confirmation("Are you sure you want to exit without saving?")?;
            if perform_action {
                return Ok(RespondResult {
                    mutated,
                    saved,
                    quit: true,
                });
            }
        }
    }
    Ok(RespondResult {
        mutated,
        saved,
        quit: false,
    })
}

fn select_players(players: &[Player], ids: Vec<IdOrName>) -> Result<Vec<&Player>, Error> {
    if players.len() == 0 {
        return Err(Error::msg(
            "error: there are addcurrently no players to select",
        ));
    }
    if ids.contains(&IdOrName::All) {
        if ids[0] == IdOrName::All && ids.len() == 1 {
            return Ok(players.iter().collect());
        } else {
            return Err(Error::msg("'all' has to be the only id."));
        }
    }
    if ids.len() == 0 {
        return Ok(vec![players.last().unwrap()]);
    }

    let ids_set: HashSet<usize> = ids
        .into_iter()
        .map(|id| match id {
            IdOrName::Id(num) => Ok(num),
            IdOrName::Name(name) => {
                players
                    .into_iter()
                    .position(|p| p.name == name)
                    .ok_or(Error::msg(format!(
                        "error: no player found with the name {name}"
                    )))
            }
            _ => panic!("unreachable"),
        })
        .collect::<Result<HashSet<usize>, Error>>()?;

    Ok(ids_set.into_iter().map(|id| &players[id]).collect())
}

fn select_players_mut(
    players: &mut [Player],
    ids: Vec<IdOrName>,
) -> Result<Vec<&mut Player>, Error> {
    if players.len() == 0 {
        return Err(Error::msg(
            "error: there are currently no players to select",
        ));
    }
    if ids.contains(&IdOrName::All) {
        if ids[0] == IdOrName::All && ids.len() == 1 {
            return Ok(players.iter_mut().collect());
        } else {
            return Err(Error::msg("'all' has to be the only id."));
        }
    }
    if ids.len() == 0 {
        return Ok(vec![players.last_mut().unwrap()]);
    }

    let ids_set: HashSet<usize> = ids
        .into_iter()
        .map(|id| match id {
            IdOrName::Id(num) => Ok(num),
            IdOrName::Name(name) => {
                players
                    .into_iter()
                    .position(|p| p.name == name)
                    .ok_or(Error::msg(format!(
                        "error: no player found with the name {name}"
                    )))
            }
            _ => panic!("unreachable"),
        })
        .collect::<Result<HashSet<usize>, Error>>()?;

    // because of the hashset, I know that the id's will be unique
    // meaning that borrowing multiple items in players as mutable is safe
    // as long as I only borrow that ones corresponding to the ids
    Ok(ids_set
        .into_iter()
        .map(|id| unsafe { &mut *(&mut players[id] as *mut Player) })
        .collect())
}

fn readline(prompt: &str) -> Result<String, String> {
    write!(std::io::stdout(), "{}", prompt).map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}

fn get_confirmation(prompt: &str) -> Result<bool, Error> {
    Ok(readline(format!("{} Y/N: ", prompt).as_str())
        .map_err(|str| Error::msg(str))?
        .trim()
        .to_lowercase()
        == "y")
}
