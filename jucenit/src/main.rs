// Error Handling
use log::trace;
use miette::Result;
// Clap
use cli::Cli;

/**
The jucenit binary entrypoint.
This main function is the first function to be executed when launching pipelight.
*/
#[tokio::main]
async fn main() -> Result<()> {
    trace!("Launch process.");
    make_handler()?;
    Cli::run().await?;
    trace!("Process clean exit.");
    Ok(())
}

/**
The make handler functions is executed right after the main function
to set up a verbose and colorful error/panic handler.
*/
pub fn make_handler() -> Result<()> {
    miette::set_panic_hook();
    Ok(())
}
