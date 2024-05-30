// Clap - command line lib
use clap::FromArgMatches;
use clap::{builder::PossibleValue, Args, Command, Parser, Subcommand, ValueHint};
// Verbosity
pub use clap_verbosity::Verbosity;
use jucenit_core::nginx::CertificateStore;
// Serde
use serde::{Deserialize, Serialize};
// Error Handling
use miette::Result;
//
use jucenit_core::{ConfigFile, JuceConfig, NginxConfig};

/*
The Cli struct is the entrypoint for command line argument parsing:
It casts arguments into the appropriate struct.

let args = Cli::from_arg_matches(&matches)

*/
#[derive(Debug, Clone, Parser)]
pub struct Cli {
    /**
    The set of subcommands.
    */
    #[command(subcommand)]
    pub commands: Commands,

    /**
     * The folowing args are global arguments available
     * for every subcommands.
     */
    /// Set a config file
    // #[arg(long, global = true, hide = true, value_name="FILE" ,value_hint = ValueHint::FilePath)]
    // pub config: Option<String>,

    /// Set verbosity level
    #[clap(flatten)]
    pub verbose: Verbosity,
}
impl Cli {
    pub fn hydrate() -> Result<()> {
        let cli = Cli::parse();
        Ok(())
    }
    pub async fn run() -> Result<()> {
        let cli = Cli::parse();
        match cli.commands {
            Commands::Push(args) => {
                if let Some(file) = args.file {
                    let config_file = ConfigFile::load(&file)?;
                    JuceConfig::push(&JuceConfig::from(&config_file)).await?;
                } else {
                    let config_file = ConfigFile::get()?;
                    JuceConfig::push(&JuceConfig::from(&config_file)).await?;
                }
            }
            Commands::Clean(args) => {
                JuceConfig::set(&JuceConfig::default()).await?;
            }
            Commands::Ssl(args) => {
                if args.renew {
                    CertificateStore::hydrate().await?;
                }
            }
            _ => {

                // Err
            }
        };
        Ok(())
    }
}

/*
An enumaration over the differen types of commands available:
*/
#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Commands {
    #[command(arg_required_else_help = true)]
    Push(File),
    #[command(arg_required_else_help = true)]
    Ssl(Ssl),
    // Developper commands
    #[command(hide = true)]
    Clean(Endpoints),
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct File {
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct Endpoints {
    #[arg(long)]
    pub ssl: bool,
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct Ssl {
    #[arg(long)]
    pub renew: bool,
    #[arg(long)]
    pub watch: bool,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::FromArgMatches;
    use clap::Parser;

    use assert_cmd::prelude::*; // Add methods on commands
    use miette::{IntoDiagnostic, Result};
    use std::process::Command; // Run commnds

    #[test]
    fn parse_command_line() -> Result<()> {
        let e = "jucenit --help";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        // println!("{:#?}", cli);
        Ok(())
    }
    #[test]
    fn push_config_file() -> Result<()> {
        let mut cmd = Command::cargo_bin("jucenit").into_diagnostic()?;
        cmd.arg("push")
            .arg("--file")
            .arg("../examples/jucenit.toml");
        cmd.assert().success();
        Ok(())
    }
}
