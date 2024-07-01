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
use jucenit_core::{ConfigFile, NginxConfig};

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
                    let config = ConfigFile::load(&file)?;
                    config.push().await?;
                } else {
                    let config = ConfigFile::get()?;
                    config.push().await?;
                }
            }
            Commands::Clean => {
                let config = ConfigFile::default();
                config.set().await?;
            }
            Commands::Edit => {
                let config = ConfigFile::pull().await?;
                config.edit().await?;
            }
            Commands::Ssl(args) => {
                if args.renew {
                    CertificateStore::hydrate().await?;
                }
                if args.clean {
                    CertificateStore::clean().await?;
                }
                if args.watch {
                    CertificateStore::watch().await?;
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
    Clean,
    Edit,
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct File {
    #[arg(long)]
    pub file: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Parser)]
pub struct Ssl {
    #[arg(long)]
    pub renew: bool,
    #[arg(long)]
    pub watch: bool,
    #[arg(long, hide = false)]
    pub clean: bool,
}

#[cfg(test)]
mod tests {
    use super::{Cli, Commands};
    use clap::FromArgMatches;
    use clap::Parser;

    use assert_cmd::prelude::*; // Add methods on commands
    use miette::{IntoDiagnostic, Result};
    use std::path::PathBuf;
    use std::process::Command; // Run commnds

    use jucenit_core::{CertificateStore, ConfigFile};

    /**
     * Set a fresh testing environment:
     * - clean certificate store
     * - set minimal nginx configuration
     */
    async fn set_testing_config() -> Result<()> {
        CertificateStore::clean().await?;

        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config = ConfigFile::load(path.to_str().unwrap())?;
        config.set().await?;

        Ok(())
    }

    fn parse_command_line() -> Result<()> {
        let e = "jucenit --help";
        let os_str: Vec<&str> = e.split(' ').collect();
        let cli = Cli::parse_from(os_str);
        println!("{:#?}", cli);
        Ok(())
    }

    #[tokio::test]
    async fn push_config_file() -> Result<()> {
        set_testing_config().await?;
        let mut cmd = Command::cargo_bin("jucenit").into_diagnostic()?;
        cmd.arg("push")
            .arg("--file")
            .arg("../examples/jucenit.toml");
        cmd.assert().success();
        Ok(())
    }

    // #[tokio::test]
    async fn renew_ssl() -> Result<()> {
        set_testing_config().await?;
        let mut cmd = Command::cargo_bin("jucenit").into_diagnostic()?;
        cmd.arg("ssl").arg("--renew");
        cmd.assert().success();
        Ok(())
    }
}
