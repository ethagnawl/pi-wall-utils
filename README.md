# pi-wall-utils

## Summary

This project is an attempt at smoothing out some of the rough edges encountered when working with an experimental PiWall setup.

There are a number of complimentary subcommands exposed by the pi-wall-utils CLI application which help in:
- dynamically creating .piwall and (virtual) .pitile config files using a (new) meta config file format
- distributing dynamically created config files to client devices

In my research, I was not able to find a common workflow which didn't involve lots of manual calculations (i.e. in the case of .piwall) or copying of files between machines using SCP (at best), FTP, USB thumbdrives (at worst), etc. for .piwall and .pitile config files.

## Status
### Generate
The meta config file is capable of being used to dynamically generate valid .piwall and .pitile config files.

It calculates the height, width and offsets (X and Y) for the top-level wall and each screen according to the dimensions (height, width and bezel) defined in the screen config blocks.

There are still some issues to be worked out around dynamic layout:

#### Bezel Compensation
I have yet to find comprehensive documentation on "bezel compensation" and it's unclear to me if/how it should affect content. As things stand, configs generated by this project will result in content being masked by bezels. This _may_ be by design but it's jarring when text or faces appear on screen and is something I want to better understand and addrress within this project. For the time being, users can still use the generated .piwall config file and manually modify the output according to their needs before syncing it to the client machines.

#### Multiple Rows
The dynamic layout provided by this project only currently supports a single row of screens. Supporting multiple rows is absolutely possible and will be addressed in the near future. The one slightly complicating factor is that the meta config will need to be made to support bezel dimensions for top, right, bottom and left bezels because, while the sides are often even, the top and bottom rarely are.

#### IDs
The generated .piwall uses explicit IDs and a config block to map client devices via their .pitile config files. This is in part because I think being explicit is preferential but also because I couldn't get dynamic hostname mapping working.

### Copy Configs
The copy config functionality is fully functional and uses SCP to distribute the files to client devices. This setup does use some conventions / make some assumptions about machine connectivitity which will be documented and potentially enhanced in the future. For example, the copy routine assumes that the client is available via an SSH alias (i.e. in ~/.ssh/config) which matches the screen's ID field in the meta config file. This workflow could be improved my making the user, hostname/IP, keyfile, etc. configurable via the meta config file. It might also be possible to support using prompted passwords but, IMO, that would be a step backwards toward a more manual workflow.

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

## Usage
### Build
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

## Future Work
