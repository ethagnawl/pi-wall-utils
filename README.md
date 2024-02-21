# pi-wall-utils

## Summary

This project is an attempt at smoothing out some of the rough edges encountered when working with an experimental PiWall setup.

There are a number of complimentary subcommands exposed by the pi-wall-utils CLI application which help in:
- dynamically creating .piwall and config files using a (new) meta config file format
- distributing config files to client devices (this operation also creates the appropriate .pitiles on the client machines)
- provisioning a new PiWall client

In my (cursory) research, I was not able to find a common workflow which didn't involve lots of manual calculations (i.e. in the case of .piwall) or copying of files between machines using SCP (at best) or USB drives (at worst) for .piwall and .pitile config files. This is also a good excuse for me to re-familiarize myself with Rust CLI applications.

## Nomenclature
This project and its documentation will use the following names to refer to the system's components:

### Server
The machine which will be broadcasting content via avconv (deprecated), ffmpeg, OBS, etc. This may or may not be a Raspberry Pi.

### Client
The Raspberry Pi(s) which will be displaying the content via pwomxplayer.

## Status

### Generate
The meta config file is capable of being used to dynamically generate functional .piwall config files -- caveats follow.

It calculates the height, width and offsets (X and Y) for the top-level wall and each screen according to the dimensions (height, width and bezel) defined in the individual screen config blocks. In theory, this simplifies the configuration workflow because it prevents the developer from having to make changes in multiple places when making modifications or adding new screens.

There are still some issues to be worked out around dynamic layout:

#### Bezel Compensation
I have yet to find comprehensive documentation on "bezel compensation" and it's unclear to me if/how it should affect content. As things stand, config files generated by this project may result in content being masked by bezels. (It's also entirely possible that my calculations are incorrect!) This _seems_ to be by design but it can be jarring when portions of text or faces are lost. This is something I want to better understand and address within this project. For the time being, users can still use the generated .piwall config file and, if necessary, manually modify the output according to their needs before copying it to the client machines.

#### Multiple Rows
The dynamic layout provided by this project only currently supports naive calculations for multiple rows -- top and bottom bezels are not currently accounted for. Properly supporting multiple rows is possible and will probably be addressed in the future. The one slightly complicating factor is that the meta config will need to be made to support dimensions for the top, right, bottom and left bezels because, while the sides are often even, the top and bottom rarely are.

#### IDs
The generated .piwall uses explicit IDs and a config block to reference client devices via their .pitile config files. This is in part because I think being explicit is preferential but also because I couldn't get dynamic hostname mapping working. These IDs are also assumed to be the hostname of the machine and the name of the associated SSH alias (documented below). Personally, I don't think this is a shortcoming but I wanted to make sure it was called out explicitly.

### Copy Configs
The copy config functionality is fully functional and uses SCP to distribute the files to client devices -- one common .piwall and custom .pitile (using the associated ID config field) each screen.

This setup does use some conventions / make some assumptions about machine connectivity which will be documented and potentially enhanced in the future. For example, the copy routine assumes that the client is available via an SSH alias which matches the screen's ID field in the meta config file. It also assumes the machines are pre-configured with passwordless logins (i.e. using pre-shared keys). This workflow could be improved my making the user, hostname/IP, keyfile, etc. configurable via the meta config file. It might also be possible to support using prompted passwords but, IMO, that would be a step backwards toward a more manual workflow.

The workflow also assumes that it's being run on the server instance and does not copy the generated .piwall to the server. However, I don't think this is a strict requirement, because the server doesn't require the .piwall config file but I will verify this next time I bring my PiWall back online.

### Provision Client
This command runs a Bash script which provisions a Pi to be used as a PiWall client. The machine is assumed to be running Debian 10. It takes arguments for a hostname and IP and does the following:
- install prerequisites
- download and install PiWall client libraries
- download test video
- configure multicast route
- set static IP
- update hostname
- create and populate .pitile

Notes:
- As things stand, this library assumes the Bash provisioning script has been made available and executable at /home/pi/scripts/provision-pi-wall-client.sh out of band. Yes, this is a hoge.
- The interface between this library and the script is admittedly wonky -- it's run using `Command` -- and it might be simpler to just run the script on its own. The output is also currently suppressed until the script exits. It might be possible and simplest to just inline the contents of the script and run it that way. This needs more thought.
- The machine will need to be rebooted after this script runs. I'd initially had the script do this and that makes sense when the script is run on its own but feels weird when it's executed from within the context of this utility library.
- The script also initially downloaded a test video and used it to launch pwomxplayer as a smoke test but that also doesn't make a lot of sense when it's run from this utility library.

## Meta Config

Sample:

```
name = "custom-wall"

[[rows]]
  [[rows.screens]]
    bezel = 1.75
    height = 16.5
    id = "pi-wall-tv-1"
    width = 22.25

  [[rows.screens]]
    bezel = 2
    height = 16.5
    id = "pi-wall-tv-2"
    width = 22.25
```

### Fields

## Commands
### Generate
### Copy Configs
### Provision

## Usage
### Build
This project is known to build on recent 64-bit versions of x86 and ARM Linux.
```
cargo build
```

### Generate
```
/path/to/pi-wall-gen generate Example.toml
```

### Copy Configs
```
/path/to/pi-wall-gen copy-config-to-clients Example.toml
```

### Provision
```
/path/to/pi-wall-gen provision 192.168.1.111 pi-wall-tv-1
```

## Future Work

### Releases
- Pi build

### CLI
- add named args
- lump config validation in main

### Misc
- use logging library and standardize logging strategy

### Generate
#### Parameterize output path
#### Support different alignments
#### Support dynamic overscan
#### Support dynamic overlap (i.e. custom arrangements)

### Copy Configs
#### Parameterize .piwall path
#### Make generate config operation an opt-in prerequisite

### Provision
#### Figure out how to distribute Bash script or inline contents

### Start command
It would be very useful if this project exposed a "start" command (among other system management commands ...) which would allow users to "start" a PiWall instance using a single command. It seems like this is conventionally done manually using SSH and I've had success using tmux/tmuxinator -- more on that below.

This workflow would likely look something like:
- SSH into clients and start listeners via pwomxplayer and any necessary / contextual (e.g. ID) flags
- SSH into server and start broadcast once clients are online and listening (broke: sleep; woke: message broker?)

As mentioned above, I've experimented with using tmuxinator for this purpose and it's worked really well. It does currently require the user to manually keep the tmuxinator config in sync with their PiWall setup but it could be possible to either dynamically create a tmuxinator config file or use a simple, common tmuxinator config file which accepts arguments and renders them using ERB. This option is also nice because tmux/tmuxinator exposes hooks which can be leveraged to handle any setup or teardown which may be required by the server or clients. I will include a link to a sample tmuxinator config file which can be used for this purpose.

One potential alternative which, IMO, could be very slick is to use my rmuxinator project as a library and dynamically start and configure a tmux session. This would be ideal because it would require fewer external dependencies (Ruby and tmuxinator) and could be managed via this project's Cargo config. This needs more thought and experimentation, though.

### The Future
- First off, the PiWall project is tremendously useful and I greatly thank the devs for sharing it with the world. However, while this setup does work, it relies on outdated versions of operating systems and libraries; the clients use a non-standard media player; its documentation leaves a lot to be desired; it's difficult to debug. (The Google group and blog posts linked below are required reading for anyone looking to set up a wall of their own.) All told, the PiWall project is 10+ years old and the ecosystem feels ... creaky. I have been wondering if this need could be better served by using modern utilities like WebRTC/RTMP and VLC. I would like to spend some time experimenting with alternatives and report back.

## Resources
- https://groups.google.com/u/1/g/piwall-users
- https://matthewepler.github.io/2016/01/05/piwall.html
- https://crt.gg/piwall
- https://github.com/Edinburgh-College-of-Art/piwall-setup?tab=readme-ov-file
- https://piwall.co.uk/
- https://pdoherty926-piwall.s3.amazonaws.com/2021-05-07-raspios-buster-10-pi-wall-client.img (Pi image created while working on this project)
- https://www.youtube.com/watch?v=RSUKVVlXrCo (CRT PiWall demo one)
- https://www.youtube.com/watch?v=NrTw9U1ad-E (CRT PiWall demo two)
