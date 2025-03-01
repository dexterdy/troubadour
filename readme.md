# Troubador

This project was created because I found that there was a distinct lack of soundscape creation tools that allowed me to use my own library of mp3s. Most of the existing tools are websites or apps that have their own collections of music and sounds. Some examples: [Syrinscape](https://syrinscape.com/), [Tabletop Audio](https://tabletopaudio.com/), [BattleBards](https://battlebards.com/). These are amazing offerings and there is good reason to use them, but if you'd rather use your own mp3's or if you want to be independant of an internet connection, this project is exactly what you need.

## State

The project currently doesn't have a GUI, only a CLI, but it does have all the basic features you'd expect from a soundscape/soundboard creation tool:

- [x] audio basics
- [x] loop sounds
  - [x] delay start (useful when you want a loop to start only later in the soundscape)
  - [ ] set loop end (useful when you want a loop to stop after a certain time)
- [x] clipping
- [ ] fades
- [x] save files
  - [x] local save files (doesn't copy your sound files)
  - [ ] sharable save files (copies your sound files)
- [x] sound grouping (apply commands to entire group at once)
- [x] separate library and interface crates
- [ ] GUI

## Usage guide

```text
$ help
troubadour: A simple audio looping application for the creation of soundscapes.

Usage: 

        add -p <PATH> -n <NAME>
                Adds a sound to the soundscape.

        remove [IDs]
                Removes sounds from the soundscape.

        show [IDs] [-g <GROUPS>]
                Shows the status and configuration of sounds.

        play [IDs] [-g <GROUPS>]
                Plays sounds.

        stop [IDs] [-g <GROUPS>]
                Stops sounds and resets the play heads to the start of each sound.

        pause [IDs] [-g <GROUPS>]
                Pauses sounds.

        volume [IDs] [-g <GROUPS>] -v <VOLUME>
                Sets the volume as a percentage. Can be higher than 100%

        loop [IDs] [-g <GROUPS>] [-d <DURATION>]
                Loops sounds at the end of their play length or DURATION, if supplied.

        unloop [IDs] [-g <GROUPS>]
                Turns of looping for these sounds.

        set-start [IDs] [-g <GROUPS>] -p <POS>
                Clips the start of sounds by selecting the starting position.

        set-end [IDs] [-g <GROUPS>] [-p <POS>]
                Clips the end of sounds by selecting the ending position. Reset by omitting POS.

        delay [IDs] [-g <GROUPS>] -d <DURATION>
                Delays playing the sound after the play command. Useful when you play multiple sounds at once.

        group [IDs] -g <GROUP>
                Adds sounds to a group. If the group doesn't exists yet, a new one will be made.

        ungroup [IDs] -g <GROUP>
                Removes sounds from a group. If the group is empty after this operation, it will be removed.

        save -p <PATH>
                Saves the current configuration to a file.

        load -p <PATH>
                Loads a saved configuration. You can choose to replace or add to current configuration.

        help
                Shows this help message.

        exit
                Exits the program.

Note that:
        - [..] indicates an optional value.
        - Most commands will select the last added sound if ID is not supplied.
        - ID can be a name or 'all'. For instance: 'play horn' or 'play all'
```
