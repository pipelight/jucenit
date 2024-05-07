// Clap - command line lib
use clap::{Parser, Subcommand, ValueHint};
// Verbosity
pub use clap_verbosity::Verbosity;
// Serde
use serde::{Deserialize, Serialize};

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
    The folowing args are global arguments available
    for every subcommands.
    */
    /// Set a config file
    #[arg(long, global = true, hide = true, value_name="FILE" ,value_hint = ValueHint::FilePath)]
    pub config: Option<String>,

    /// Set verbosity level
    #[clap(flatten)]
    pub verbose: Verbosity,
}

/*
An enumaration over the differen types of commands available:
- PreCommand that only needs a partial env to run,
- PostCommands that needs the full env to be loaded to run.
*/
#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Commands {
    Adapt,
}

