// Error Handling
use miette::{Error, IntoDiagnostic, Result};
// Watch files and Tcp
use http::uri::Uri;
use watchexec::Watchexec;
use watchexec_events::{
    filekind::{AccessKind, AccessMode, FileEventKind},
    Event, Tag,
};
use watchexec_signals::Signal;

use std::io::{BufRead, BufReader, BufWriter, Read};
use std::net::TcpStream;
use std::path::Path;
// Globals
use super::SETTINGS;
// Structs
use super::Nginx;

fn main() -> Result<()> {
    // Automatically select the best implementation for your platform.

    // Add a path to be watched. All files and directories at that path and
    // below will be monitored for changes.

    Ok(())
}

impl Nginx {
    pub async fn listen_config() -> Result<()> {
        let settings = SETTINGS.lock().unwrap().clone();
        let state_dir = settings.state_dir.unwrap();
        let wx = Watchexec::new(|mut action| {
            for event in action.events.iter() {
                // Print any events
                eprintln!("EVENT: {event:?}");
                // If write to file and close, then the config is concidered to have changed
                if event
                    .tags
                    .contains(&Tag::FileEventKind(FileEventKind::Access(
                        AccessKind::Close(AccessMode::Write),
                    )))
                {
                    // If a new domain has been added and has no ssl certificate
                }
            }
            // if Ctrl-C is received, quit
            if action.signals().any(|sig| sig == Signal::Interrupt) {
                action.quit();
            }
            action
        })?;

        // watch nginx main configuration file
        wx.config.pathset([state_dir + "/conf.json"]);

        let _ = wx.main().await.into_diagnostic()?;
        Ok(())
    }
    pub fn listen_certs() -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use miette::Result;

    // #[tokio::test]
    async fn listen_tcp() -> Result<()> {
        Nginx::listen_config().await?;
        Ok(())
    }
}
