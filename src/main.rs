use ini::Ini;
use serde::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

extern crate toml;

#[derive(Debug, Deserialize)]
pub struct Screen {
    id: String,
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

fn parse_config_file() -> Result<Config, String> {
    let mut config_file = match File::open("Example.toml") {
        Ok(file) => file,
        Err(_) => return Err(String::from("Unable to open config file.")),
    };
    let mut contents = String::new();

    match config_file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(_) => return Err(String::from("Unable to read config file.")),
    }

    println!("contents: {:#?}", contents);

    let decoded = toml::from_str(&contents);
    println!("decoded: {:#?}", decoded);

    match decoded {
        Ok(config) => Ok(config),
        Err(error) => Err(error.to_string()),
    }
}

fn copy_configs_to_clients() {
    // accept config
    for client in ["fossil"] {
        // TODO: abstract
        println!("Copying .piwall to : {}", client);
        let mut shell = Command::new("sh");
        let output = shell
            .arg("-c")
            .arg(format!("scp .piwall {}:/home/peter/Downloads", client))
            .status()
            .expect("bar");
        println!("debug: {:#?}", output);

        // TODO: abstract
        println!("Copying .pitile to : {}", client);
        let mut conf = Ini::new();
        conf.with_section(Some("tile")).set("id", client);
        conf.write_to_file(".pitile").unwrap();

        let mut shell = Command::new("sh");
        let output = shell
            .arg("-c")
            .arg(format!("scp .pitile {}:/home/peter/Downloads", client))
            .status()
            .expect("bar");
        println!("debug: {:#?}", output);
    }
}

fn generate_piwall_config(config: Config) {
    let mut wall_height = 0.0;
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
    let wall_id = config.name;

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

    conf.with_section(Some(format!("{}_config", wall_id.clone())))
        .set("pi1", "default");
    for (i, row) in config.rows.iter().enumerate() {
        for (ii, screen) in row.screens.iter().enumerate() {
            conf.section_mut(Some(format!("{}_config", wall_id.clone())))
                .unwrap()
                .insert(format!("pi{}", (i + 1) + ii), screen.id.clone());
        }
    }

    println!("conf: {:#?}", conf);

    // TODO: alias? backup any existing piwalls?
    conf.write_to_file(".piwall").unwrap();
}

fn main() {
    let config = parse_config_file();
    assert_eq!(config.is_ok(), true);
    let config_ = config.unwrap();
    generate_piwall_config(config_);
    copy_configs_to_clients()
}
