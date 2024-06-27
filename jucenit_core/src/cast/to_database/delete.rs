// Database
use crate::database::{connect_db, fresh_db};
use crate::{ConfigFile, ConfigUnit, NginxConfig};
// Sea orm
// use indexmap::IndexMap;
use crate::database::entity::{prelude::*, *};
use migration::{Migrator, MigratorTrait};
use sea_orm::{
    prelude::*, query::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase,
};
use sea_orm::{Database, DatabaseConnection};
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
}
impl ConfigUnit {
    pub async fn remove_from_db(&self) -> Result<()> {
        let unit = self;
        let db = connect_db().await?;

        let match_ = NgMatch::find()
            .filter(Condition::all().add(ng_match::Column::Uuid.eq(&unit.uuid)))
            .one(&db)
            .await
            .into_diagnostic()?;
        let match_ = match_.unwrap();

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

        // Delete action if not linked to other matches.
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
            action.delete(&db).await.into_diagnostic()?;
        }

        match_.delete(&db).await.into_diagnostic()?;

        Ok(())
    }
}
#[cfg(test)]
mod test {
    use crate::database::entity::{prelude::*, *};
    use crate::database::{connect_db, fresh_db};
    use crate::{ConfigFile, Match, Nginx, NginxConfig};
    use sea_orm::{prelude::*, sea_query::OnConflict, ActiveValue, InsertResult, MockDatabase};
    // Logging
    use tracing::{debug, Level};
    // Error Handling
    use miette::{IntoDiagnostic, Result};

    async fn set_default_config() -> Result<()> {
        let db = fresh_db().await?;
        // Get struct from config
        let toml = "
        [[unit]]
        uuid = 'd3630938-5851-43ab-a523-84e0c6af9eb1'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com', 'example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'

        [[unit]]
        uuid = '70372c19-cb64-4f18-9c54-7bac10112b95'
        listeners = ['*:443']
        [unit.match]
        hosts = ['test.com']
        [unit.action]
        proxy = 'http://127.0.0.1:1337'

        [[unit]]
        uuid = 'd462482d-21f7-48d6-8360-528f9e664c2f'
        listeners = ['*:443']
        [unit.match]
        uri = ['/home']
        [unit.action]
        proxy = 'http://127.0.0.1:8333'

        [[unit]]
        uuid = 'cc4e626a-9354-480e-a78b-f9f845148984'
        listeners = ['*:443']
        [unit.match]
        hosts = ['api.example.com']
        [unit.action]
        proxy = 'http://127.0.0.1:8222'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        config.push().await?;
        let nginx_config = NginxConfig::pull().await?;
        println!("{:#?}", nginx_config);
        Ok(())
    }

    #[tokio::test]
    async fn remove_unit_by_uuid() -> Result<()> {
        set_default_config().await?;
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
