use clap::Arg;
use clap::{App, AppSettings, SubCommand};
use ini::Ini;
use serde::Deserialize;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::str::FromStr;
use tempfile::NamedTempFile;

extern crate toml;

pub fn parse_args<I, T>(args: I) -> CliArgs
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let project_config_file_arg = Arg::with_name("PROJECT_CONFIG_FILE")
        .help("The path to the project config file")
        .required(true);
    let app_matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            SubCommand::with_name("copy-config-to-clients")
                .about("Copy piwall config to clients using toml")
                .arg(&project_config_file_arg),
        )
        .subcommand(
            SubCommand::with_name("generate")
                .about("Generate a piwall config using toml")
                .arg(&project_config_file_arg),
        )
        .subcommand(
            SubCommand::with_name("start")
                .about("Start a tmux session using a path to a project config file")
                .arg(&project_config_file_arg),
        )
        .get_matches_from(args);

    let (command_name, command_matches) = match app_matches.subcommand() {
        (name, Some(matches)) => (name, matches),
        (_, None) => {
            panic!("Subcommand should be forced by clap");
        }
    };

    let command = match CliCommand::from_str(command_name) {
        Ok(command) => command,
        Err(error) => {
            panic!("{}", error.to_string());
        }
    };

    let project_name = command_matches
        .value_of("PROJECT_CONFIG_FILE")
        .expect("project file is required by clap")
        .to_string();

    CliArgs {
        command,
        project_name,
    }
}

#[derive(Debug, PartialEq)]
pub enum CliCommand {
    CopyConfigToClients,
    Generate,
    Start,
}

#[derive(Debug)]
pub struct ParseCliCommandError;

// TODO: this boilerplate can be cut down by using a third-party crate
impl fmt::Display for ParseCliCommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Missing implementation for subcommand, please file a bug report"
        )
    }
}

impl Error for ParseCliCommandError {}

impl FromStr for CliCommand {
    type Err = ParseCliCommandError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "copy-config-to-clients" => Ok(Self::CopyConfigToClients),
            "generate" => Ok(Self::Generate),
            "start" => Ok(Self::Start),
            // This should only ever be reached if subcommands are added to
            // clap and not here
            _ => Err(ParseCliCommandError),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct CliArgs {
    pub command: CliCommand,
    pub project_name: String,
}

#[derive(Debug, Deserialize)]
pub struct Screen {
    id: String,
    // TODO: introduce explict hostname for copying
    // TODO: introduce dulr bezel dims
    bezel: f32,
    height: f32,
    width: f32,
}

#[derive(Debug, Deserialize)]
pub struct Row {
    screens: Vec<Screen>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    name: String,
    rows: Vec<Row>,
}

fn parse_config_file(config_file_path: &String) -> Result<Config, String> {
    let mut config_file = match File::open(config_file_path) {
        Ok(file) => file,
        Err(_) => return Err(String::from("Unable to open config file.")),
    };
    let mut contents = String::new();

    match config_file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err(String::from("Unable to read config file.")),
    }

    let decoded = toml::from_str(&contents);
    println!("parsed config file: {:#?}", decoded);

    match decoded {
        Ok(config) => Ok(config),
        Err(error) => Err(error.to_string()),
    }
}

fn copy_pi_wall_to_client(
    id: &String,
    local_pi_wall_path: &String,
    remote_piwall_path: &String,
) -> Result<(), String> {
    let mut shell = Command::new("sh");
    let output = shell
        .arg("-c")
        .arg(format!(
            "scp {local_pi_wall_path} {id}:{remote_piwall_path}"
        ))
        .status()
        .expect("Could not copy {local_config_path} to: {id}");

    match output.code() {
        Some(0) => {
            println!("Copying {local_pi_wall_path} to {id} succeeded.");
        }
        Some(code) => {
            return Err(format!(
                "Copying {local_pi_wall_path} to {id} failed with status code: {code}."
            ));
        }
        None => {
            return Err(String::from(
                "Copying {local_config_path} to {id} failed when process was terminated by signal",
            ));
        }
    }

    println!("copy piwall config output: {:#?}", output);

    Ok(())
}

fn copy_configs_to_clients(config: &Config) -> Result<(), String> {
    // TODO: parameterize?
    let local_piwall_config_path = String::from(".piwall");
    let remote_piwall_config_path = String::from(format!("~/.piwall"));
    let remote_tile_config_path = String::from(format!("~/.pitile"));

    for row in config.rows.iter() {
        for client in row.screens.iter() {
            let id = client.id.clone();
            println!("copy {local_piwall_config_path} to: {id}:{remote_piwall_config_path}");

            let copy_pi_wall_to_client_result =
                copy_pi_wall_to_client(&id, &local_piwall_config_path, &remote_piwall_config_path);
            // TODO: return application error
            assert_eq!(copy_pi_wall_to_client_result.is_ok(), true);

            let local_pitile = NamedTempFile::new()
                .map_err(|error| format!("CopyConfigToClients error: {}", error));

            // NOTE: unwrapping when setting local_pi_tile_path results in
            // the temp file not being deleted after the scope exits!
            let local_pitile_ = local_pitile.unwrap();
            let local_pi_tile_path = local_pitile_.path().to_string_lossy().to_string();
            println!("local_pi_tile_path: {:#?}", local_pi_tile_path);

            // TODO: abstract
            let mut conf = Ini::new();
            conf.with_section(Some("tile")).set("id", id.clone());
            conf.write_to_file(&local_pi_tile_path).unwrap();

            let mut shell = Command::new("sh");
            let output = shell
                .arg("-c")
                .arg(format!(
                    "scp {local_pi_tile_path} {id}:{remote_tile_config_path}"
                ))
                // TODO: fail hard and fast using patterns established above
                .status()
                .expect("Could not copy .pitile to: {id}");

            match output.code() {
                Some(0) => {
                    println!("Copying {local_pi_tile_path} to {id} succeeded.");
                }
                Some(code) => {
                    return Err(format!(
                        "Copying {local_pi_tile_path} to {id} failed with status code: {code}."
                    ));
                }
                None => {
                    return Err(String::from("Copying {local_pi_tile_path} to {id} failed when process was terminated by signal"));
                }
            }
        }
    }
    Ok(())
}

fn generate_piwall_config(config: &Config, output_path: Option<&str>) {
    let default_output_path = String::from(".piwall");
    let output_path_ = output_path.unwrap_or(&default_output_path);
    // TODO: this needs to take into account top/bottom bezels
    let mut wall_height = 0.0;
    // TODO: is this correct? should be bezel+width + bezel+width for each screen in row
    let mut wall_width = 0.0;

    for row in config.rows.iter() {
        let mut row_height = 0.0;
        let mut row_width = 0.0;
        for screen in row.screens.iter() {
            row_width += screen.width;
            if screen.height > row_height {
                row_height = screen.height
            }
        }

        if row_width > wall_width {
            // TODO: does this make sense? would it be better to use the min or
            // do this on a row-by-row basis?
            wall_width = row_width
        }

        wall_height += row_height
    }

    let mut conf = Ini::new();
    let wall_id = config.name.clone();

    conf.with_section(Some(wall_id.clone()))
        .set("height", wall_height.to_string())
        .set("width", wall_width.to_string())
        .set("x", "0")
        .set("y", "0");

    for row in config.rows.iter() {
        let mut offset = 0.0;
        for (ii, screen) in row.screens.iter().enumerate() {
            let offset_ = if ii == 0 { 0.0 } else { offset + screen.bezel };
            conf.with_section(Some(screen.id.clone()))
                .set("height", screen.height.to_string())
                .set("wall", wall_id.clone())
                .set("width", screen.width.to_string())
                .set("x", offset_.to_string())
                // TODO: compute using previous row height
                // how should row height be computed? min? max? should it
                // become a field?
                .set("y", "0");
            offset += offset_ + screen.width + screen.bezel
        }
    }

    let conf_key = Some(format!("{}_config", wall_id.clone()));
    // create empty config block
    // see: https://github.com/zonyitoo/rust-ini/issues/71
    conf.entry(conf_key.clone()).or_insert(Default::default());

    for row in config.rows.iter() {
        for screen in row.screens.iter() {
            conf.section_mut(conf_key.clone())
                .unwrap()
                .insert(screen.id.clone(), screen.id.clone());
        }
    }

    println!("generated piwall config: {:#?}", conf);

    conf.write_to_file(output_path_).unwrap();

    println!("wrote piwall config to: {:#?}", output_path_);
}

fn start(_config: &Config) {
    // TODO: everything!
    // presumably this would:
    // - ssh into the clients and start the listeners
    // - ssh into the controller and start the broadcast
    //   -- what if it's being run on the controller?
    // is this a good idea? is there a clean way of managing the various
    // sessions? i was previously using tmuxinator to do this and it worked
    // very nicely, so maybe we could use rmuxinator as a library to achieve
    // a similar workflow? that prevents this application from having to do
    // anything clever but introduces another dependency, assumes folks are
    // familiar with tmux, etc..
    println!("start fn has not been implemented.");
}

fn main() -> Result<(), String> {
    let cli_args = parse_args(env::args_os());
    let config = parse_config_file(&cli_args.project_name);

    assert_eq!(config.is_ok(), true);
    let config_ = config.unwrap();

    match cli_args.command {
        CliCommand::Generate => {
            // TODO: move output_path into clap config and define default (.piwall)
            // TODO: .map_err(|error| format!("Application error: {}", error)),
            generate_piwall_config(&config_, Some(".piwall"));
            Ok(())
        }
        CliCommand::CopyConfigToClients => {
            // TODO: move input_path into clap config and define default (.piwall)
            copy_configs_to_clients(&config_)
                .map_err(|error| format!("CopyConfigToClients error: {}", error))
        }
        CliCommand::Start => {
            start(&config_);
            Ok(())
        }
    }
}
