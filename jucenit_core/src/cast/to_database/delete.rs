// Database
use crate::database::{connect_db, fresh_db};
use crate::{ConfigFile, ConfigUnit, NginxConfig};
// Sea orm
// use indexmap::IndexMap;
use crate::database::entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult};
use sea_orm::{Database, DatabaseConnection};
use serde_json::json;
// Logging
use tracing::{debug, Level};
// Error Handling
use miette::{Error, IntoDiagnostic, Result, WrapErr};

impl ConfigFile {
    pub async fn remove(&self) -> Result<()> {
        self.remove_from_db().await?;
        let nginx_config = NginxConfig::pull().await?;
        nginx_config.set().await?;
        Ok(())
    }
    pub async fn remove_from_db(&self) -> Result<()> {
        for unit in &self.unit {
            unit.remove_from_db().await?;
        }
        Ok(())
    }
    pub async fn purge_http_challenge() -> Result<()> {
        let db = connect_db().await?;

        let like = json!({"uri":[]});
        let like = like.to_string();

        let matches = NgMatch::find().all(&db).await.into_diagnostic()?;
        let challenges: Vec<&ng_match::Model> = matches
            .iter()
            .filter(|x| {
                if let Some(raw_params) = x.raw_params.clone() {
                    let raw_params: serde_json::Value = serde_json::from_str(&raw_params).unwrap();
                    return raw_params["uri"]
                        .to_string()
                        .contains("/.well-known/acme-challenge/");
                }
                return false;
            })
            .collect();

        for x in challenges {
            let x = x.clone();
            x.delete(&db).await.into_diagnostic()?;
        }
        Ok(())
    }
}
impl ConfigUnit {
    pub async fn remove(&self) -> Result<()> {
        self.remove_from_db().await?;
        let nginx_config = NginxConfig::pull().await?;
        nginx_config.set().await?;
        Ok(())
    }
    pub async fn remove_from_db(&self) -> Result<()> {
        let unit = self;
        let db = connect_db().await?;

        let match_ = NgMatch::find()
            .filter(Condition::all().add(ng_match::Column::Uuid.eq(&unit.uuid)))
            .one(&db)
            .await
            .into_diagnostic()?;

        if let Some(match_) = match_ {
            let hosts = match_.find_related(Host).all(&db).await.into_diagnostic()?;
            for host in hosts {
                // Delete host if not linked to other matches.
                if host
                    .find_related(NgMatch)
                    .filter(
                        Condition::all()
                            .not()
                            .add(ng_match::Column::Uuid.eq(&unit.uuid)),
                    )
                    .all(&db)
                    .await
                    .into_diagnostic()?
                    .is_empty()
                {
                    host.delete(&db).await.into_diagnostic()?;
                }
            }
            let action = match_
                .find_related(Action)
                .one(&db)
                .await
                .into_diagnostic()?;
            let action = action.unwrap();

            let listeners = match_
                .find_related(Listener)
                .all(&db)
                .await
                .into_diagnostic()?;
            for listener in listeners {
                // Delete listeners if no related match
                if listener
                    .find_related(NgMatch)
                    .filter(
                        Condition::all()
                            .not()
                            .add(ng_match::Column::Uuid.eq(&unit.uuid)),
                    )
                    .all(&db)
                    .await
                    .into_diagnostic()?
                    .is_empty()
                {
                    listener.delete(&db).await.into_diagnostic()?;
                }
            }

            // Delete action if not linked to other matches.
            let mut del_action = false;
            if action
                .find_related(NgMatch)
                .filter(
                    Condition::all()
                        .not()
                        .add(ng_match::Column::Uuid.eq(&unit.uuid)),
                )
                .all(&db)
                .await
                .into_diagnostic()?
                .is_empty()
            {
                del_action = true;
            }

            match_.delete(&db).await.into_diagnostic()?;
            // Delete action after match (fk constraint)
            if del_action {
                action.delete(&db).await.into_diagnostic()?;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
mod test {
    use crate::database::entity::{prelude::*, *};
    use crate::database::{connect_db, fresh_db};
    use crate::{ConfigFile, ConfigUnit, Match, Nginx, NginxConfig};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
    use std::path::PathBuf;
    // Logging
    use tracing::{debug, Level};
    // Error Handling
    use miette::{IntoDiagnostic, Result};

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
    async fn remove_http_challenges() -> Result<()> {
        set_testing_config().await?;
        let toml = "
        uuid = 'random-uuid'
        listeners = ['*:80']
        [match]
        hosts = ['test.com']
        uri = ['/.well-known/acme-challenge/uuid']
        [action]
        share = ['/tmp/jucenit/challenge_uuid.txt']
        ";
        let unit = ConfigUnit::from_toml_str(&toml)?;
        unit.push().await?;

        ConfigFile::purge_http_challenge().await?;

        Ok(())
    }
    #[tokio::test]
    async fn remove_unit_by_uuid() -> Result<()> {
        set_testing_config().await?;
        let toml = "
        [[unit]]
        uuid = 'd3630938-5851-43ab-a523-84e0c6af9eb1'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com', 'example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        config.remove().await?;
        // let nginx_config = NginxConfig::pull().await?;
        // println!("{:#?}", nginx_config);
        Ok(())
    }
}
