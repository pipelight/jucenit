// Clap - command line lib
use clap::{Parser, Subcommand, ValueHint};
// Verbosity
pub use clap_verbosity::verbosity::internal::InternalVerbosity;
// Serde
use serde::{Deserialize, Serialize};
// Struct
mod post;
mod pre;
pub use post::*;
pub use pre::*;

/*
The Cli struct is the entrypoint for command line argument parsing:
It casts arguments into the appropriate struct.

let args = Cli::from_arg_matches(&matches)

*/
#[derive(Debug, Clone, Parser, PartialEq)]
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
    // #[serde(flatten)]
    pub verbose: Verbosity,

    #[clap(flatten)]
    // #[serde(flatten)]
    pub internal_verbose: InternalVerbosity,

    /// Pass those arguments to deno
    #[arg(global = true, last = true, allow_hyphen_values = true)]
    pub raw: Option<Vec<String>>,
}

/*
Why this and not a simple boolean?
Clap interprets Option<bool> as bool.
This enum is a workaround.
It can be either None, Some("true") or Some("false")

Then it is possible to know if the flag has been used
on command line.

*/
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Attach {
    True,
    False,
}

/*
An enumaration over the differen types of commands available:
- PreCommand that only needs a partial env to run,
- PostCommands that needs the full env to be loaded to run.
*/
#[derive(Debug, Clone, Eq, PartialEq, Subcommand)]
pub enum Commands {
    #[clap(flatten)]
    PreCommands(PreCommands),
    #[clap(flatten)]
    PostCommands(PostCommands),
}
