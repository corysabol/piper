#![allow(unused)]

use clap::{Parser, Subcommand};
use config::Config;
use piper_agent::{agent::Agent, *};
use piper_runner::*;
use piper_dsl::Pipeline;
use serde::Deserialize;
use std::{collections::HashMap, fs, path::Path};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    cmd: SubCommand,
}

// Subcommands
#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Initialize a project directory or agent with a config file
    Init {
        /// Path to a directory to initialize typically .
        #[clap(short, long, parse(from_os_str), default_value = ".")]
        path: std::path::PathBuf,
        /// Emit an agent config file instead
        #[clap(short, long)]
        agent: bool,
        /// Whether or not to interactively initialize TODO
        #[clap(short, long)]
        interactive: bool,
    },
    /// Run a pipeline
    Run {
        /// Path to a pipeline file
        #[clap(short, long, parse(from_os_str))]
        path: std::path::PathBuf,
        /// Whether the pipeline should be ran against a remote agent
        #[clap(short, long)]
        remote: bool,
        /// IP and port or name of a remote agent --remote must be specified for this to be used
        #[clap(short, long, default_value = "http://127.0.0.1:50051")]
        agent: String,
        /// Force regeneration of meta-pipeline (only applies to meta-pipelines)
        #[clap(short, long)]
        regenerate: bool,
    },
    /// Start in agent mode
    StartAgent {
        // Start in agent mode
        #[clap(long, default_value = "127.0.0.1:50051")]
        agent_listen_addr: String,
        #[clap(long)]
        auth_key: Option<String>,
    },
    /// Manage remote agents
    Agents {},
    /// Check the status of a remote pipeline
    Status {},
}

// Representation of an agent
// TODO: Figure out if this is also where I should configure authz and encryption
//struct Agent {
//    listen_addr: String,
//}

// Hold to two config types
// enum Config {
//     Project(ProjectConfig),
//     Agent(AgentConfig),
// }

#[derive(Debug)]
struct ConfigError;

impl std::error::Error for ConfigError {}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "There was an error loading the config file")
    }
}

#[derive(Default, Debug, Deserialize)]
struct ProjectConfig {
    owner: Option<String>,
    description: Option<String>,
    auth_key: Option<String>,
    agents: Vec<AgentConfig>,
}

#[derive(Default, Debug, Deserialize)]
struct AgentConfig {
    name: Option<String>,
    ip: String,               // required
    auth_key: Option<String>, // required
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Include an example pipeline
    // TODO

    // Include an example client / project config file
    let example_project_config = include_str!("../config.toml");
    // Include an example agent config file
    let example_agent_config = include_str!("../agent_config.toml");

    // Create a config struct and check if the file is present
    // if it is then
    // Slurp up the config file
    let config = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();

    // println!("{:?}", config);

    let args = Args::parse();
    match args.cmd {
        SubCommand::Init {
            path,
            agent,
            interactive,
        } => {
            // Handle emitting various config files
            if agent {
                println!("Emitting agent config example");
                // emit agent config
                fs::write(path.join("config.toml"), example_agent_config);
            } else {
                println!("Emitting project config example");
                fs::write(path.join("config.toml"), example_project_config);
            }
        }
        SubCommand::Run {
            path,
            remote,
            agent,
            regenerate,
        } => {
            // if remote flag is present run against
            // a given remote agent ip
            if remote {
                // use the agent value to look up the agent IP in the config struct
                // if it's not found then attempt to use the value supplied as the addr
                client::client_run(agent, path).await?;
            } else {
                // otherwise run the pipeline locally using the runner
                runner::run_from_file_with_options(path, regenerate);
            }
        }
        SubCommand::StartAgent {
            auth_key,
            agent_listen_addr,
        } => {
            // Load the config file
            // Start the gRPC agent
            // TODO: Pass constructed config struct to agent start function
            agent::start_agent(agent_listen_addr).await?;
        }
        SubCommand::Agents {} => todo!(),
        SubCommand::Status {} => todo!(),
    }

    Ok(())
}
