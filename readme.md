# Troubador

This project was created because I found that there was a distinct lack of soundscape creation tools that allowed me to use my own library of mp3s. Most of the existing tools are websites or apps that have their own collections of music and sounds. Some examples: [Syrinscape](https://syrinscape.com/), [Tabletop Audio](https://tabletopaudio.com/), [BattleBards](https://battlebards.com/). These are amazing offerings and there is good reason to use them, but if you'd rather use your own mp3's or if you want to be independant of an internet connection, this project is exactly what you need.

## State

The project currently doesn't have a GUI, only a CLI, but it does have all the basic features you'd expect from a soundscape/soundboard creation tool:

- [x] basics
  - [x] play
  - [x] pause
  - [x] stop
  - [x] volume
- [-] loop sounds
  - [x] loop at the end of a sound
  - [x] loops longer than sound length (adds silence)
  - [x] loops shorter than sound length (clips sound)
  - [x] delay start (useful when you want a loop to start only later in the soundscape)
  - [ ] set loop end (useful when you want a loop to stop after a certain time)
- [x] clipping
  - [x] clip start
  - [x] clip end
- [ ] fades (this will be a simple toggle)
- [-] save files
  - [x] local save files (doesn't copy your sound files)
  - [ ] sharable save files (copies your sound files)
  - [x] add save file to current soundscape
- [x] sound grouping (apply commands to entire group at once)
- [ ] GUI

## Usage guide

```text
$ --help
dnd-player: A simple audio looping application for the creation of soundscapes.

Usage:
        add -p <PATH> -n <NAME>         Will add a sound to the soundscape.
        remove [IDs]                    Will remove a sound from the soundscape.
        show [IDs]                      Will show the status of a sound.
        play [IDs]
        stop [IDs]
        pause [IDs]
        volume [IDs] -v <VOLUME>        Set volume as a percentage. Can be higher than 100%
        loop [IDs] [-d <DURATION>]      Will loop at the end of the sound or the DURATION, if supplied.
        unloop [IDs]                    Turns of looping for this sound.
        set-start [IDs] -p <POS>        Clips the start of a sound by selecting the starting position.
        set-end [IDs] [-p <POS>]        Clips the end of a sound by selecting the ending position. Reset by omitting POS.
        delay [IDs] -d <DURATION>       Delays playing the sound after the play command. Useful in combination with  play-all.
        help                            Shows this help message.
        exit                            Exits the program.

Note that:
        - [..] indicates an optional value.
        - Most commands will select the last added sound if ID is not supplied.
        - ID can be a name, 'all', or an index. For instance: 'play horn', 'play all' or 'play 0'
```
