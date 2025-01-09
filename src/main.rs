mod server;
mod generator;

use clap::Command;
use std::fs::{self};
use std::string::ToString;
use superheap::types::Config;

fn main() {
    let config_arg = clap::Arg::new("config-path")
        .short('c')
        .long("config")
        .value_name("FILE")
        .help("Path to config file");

    let debug_arg = clap::Arg::new("debug")
        .short('d')
        .long("debug")
        .action(clap::ArgAction::SetTrue)
        .help("Enable debug mode");

    let cmd = Command::new("superheap")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("serve").about("Start the SMTP server").arg(config_arg.clone()).arg(debug_arg.clone()),
            Command::new("generate").about("Generate the RSS file").arg(config_arg.clone()).arg(debug_arg.clone()),
        ]);
    let matches = cmd.get_matches();

    let sub_cmd = matches.subcommand().unwrap();
    let config_path = sub_cmd.1.get_one::<String>("config-path");
    let is_debug = sub_cmd.1.get_one::<bool>("debug").unwrap_or(&false);

    let config = match config_path {
        Some(path) => {
            let f = fs::File::open(path).expect("Failed to open config file");
            let config: Config = serde_json::from_reader(f).expect("Failed to parse config file");
            config
        },
        None => {
            eprintln!("No config file provided. Choosing default config.");
            let config: Config = Config::new(
                10025,
                "receiver@example.com".to_string(),
                "Test Feed".to_string(),
                "test".to_string(),
                "FNU Author".to_string(),
                "/tmp/superheap.db".to_string(),
                5,
                "/tmp/superheap/".to_string()
            );
            config
        }
    };

    match matches.subcommand() {
        Some(("serve", _)) => {
            server::start_mail_server(config, *is_debug);
        }
        Some(("generate", _)) => {
            generator::generate_feeds(config).expect("Failed to generate feeds");
        }
        _ => unreachable!("Subcommand required by clap"),
    };

}
