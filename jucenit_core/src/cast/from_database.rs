// Database
use super::config;
use crate::database::{connect_db, fresh_db};
use crate::{ConfigFile, ConfigUnit, NginxConfig};
// Sea orm
// use indexmap::IndexMap;
use crate::database::entity::{prelude::*, *};
use rayon::iter::Update;
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase,
};
use sea_orm::{Database, DatabaseConnection};
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

// Fs
use std::env;
use std::process::{Command, Stdio};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

impl ConfigFile {
    pub async fn pull() -> Result<Self> {
        let db = connect_db().await?;

        let mut config = ConfigFile::default();

        let matches = NgMatch::find().all(&db).await.into_diagnostic()?;

        for match_ in matches {
            let action = match_
                .find_related(Action)
                .one(&db)
                .await
                .into_diagnostic()?;
            let hosts = match_.find_related(Host).all(&db).await.into_diagnostic()?;
            let listeners = match_
                .find_related(Listener)
                .all(&db)
                .await
                .into_diagnostic()?;

            let unit = ConfigUnit {
                uuid: match_.clone().uuid,
                action: Some(config::Action::from(&action.unwrap())),
                match_: config::Match::from(&match_, hosts),
                listeners: listeners.iter().map(|x| x.clone().ip_socket).collect(),
                ..Default::default()
            };

            config.unit.push(unit);
        }
        Ok(config)
    }
    pub async fn edit(&self) -> Result<()> {
        let tmp_dir = "/tmp/jucenit";
        fs::create_dir_all(tmp_dir).await.into_diagnostic()?;
        let path = "/tmp/jucenit/jucenit.config.tmp.toml".to_owned();

        // Retrieve config
        let toml = ConfigFile::pull().await?.to_toml()?;
        // Create and write to file
        let mut file = fs::File::create(path.clone()).await.into_diagnostic()?;
        let bytes = toml.as_bytes();
        file.write_all(bytes).await.into_diagnostic()?;

        // Modify file with editor
        let editor = env::var("EDITOR").into_diagnostic()?;
        let child = Command::new(editor)
            .arg(path.clone())
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .expect("Couldn't spawn a detached subprocess");
        let output = child.wait_with_output().into_diagnostic()?;

        // Try Update nginx-unit config
        let tmp_config = ConfigFile::load(&path)?;
        tmp_config.set().await?;

        // Clean up tmp files before exit
        fs::remove_file(path).await.into_diagnostic()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::database::entity::{prelude::*, *};
    use crate::database::{connect_db, fresh_db};
    use crate::{ConfigFile, Match, NginxConfig};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
    // Logging
    use tracing::{debug, Level};
    // Error Handling
    use miette::{IntoDiagnostic, Result};
    use std::path::PathBuf;
    /**
     * Set a fresh testing environment:
     * - clean certificate store
     * - set minimal nginx configuration
     */
    async fn set_testing_config() -> Result<()> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("../examples/jucenit.toml");

        let config = ConfigFile::load(path.to_str().unwrap())?;
        config.set().await?;

        Ok(())
    }

    #[tokio::test]
    async fn get_config() -> Result<()> {
        set_testing_config().await?;
        ConfigFile::pull().await?;
        Ok(())
    }
}
