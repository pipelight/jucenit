/**
The jucenit binary entrypoint.
This main function is the first function to be executed when launching pipelight.
*/
fn main() -> Result<()> {
    trace!("Launch process.");
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
