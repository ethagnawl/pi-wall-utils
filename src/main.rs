use ini::Ini;
use std::process::Command;

fn write_piwall_to_file() {
    let mut conf = Ini::new();

    // [wall]
    // width=1067
    // height=613
    // x=0
    // y=0

    // [pi1]
    // wall=wall
    // width=522
    // height=293
    // x=0
    // y=0

    // [config]
    // pi1=pi1

    conf.with_section(Some("wall"))
        .set("height", "40")
        .set("width", "40")
        .set("x", "0")
        .set("y", "0");
    conf.with_section(Some("pi_wall_tv_0"))
        .set("height", "20")
        .set("wall", "wall")
        .set("width", "20")
        .set("x", "0")
        .set("y", "0");
    conf.with_section(Some("pi_wall_tv_1"))
        .set("height", "20")
        .set("wall", "wall")
        .set("width", "20")
        .set("x", "40")
        .set("y", "0");
    conf.with_section(Some("config"))
        .set("pi1", "pi_wall_tv_0")
        .set("pi2", "pi_wall_tv_1");
    conf.write_to_file(".custom.piwall").unwrap();
    println!("Done!")
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

fn main() {
    // TODO: parse config
    write_piwall_to_file();
    copy_configs_to_clients()
}
