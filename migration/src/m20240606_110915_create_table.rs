//!
//! Generate entities.
//!
//! ```sh
//! # on the repo root
//! sea-orm-cli generate entity --output-dir ./entity/src
//! ```
//!

use miette::{IntoDiagnostic, Result};
use sea_orm_migration::prelude::*;
use strum::EnumIter;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // todo!();
        manager
            .create_table(
                Table::create()
                    .table(Host::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Host::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Host::Domain).string())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Match::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Match::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Match::Uri).string())
                    .col(ColumnDef::new(Match::Source).string())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Listener::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Listener::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Listener::IpSocket).string())
                    // .col(ColumnDef::new(Listener::Tls).string())
                    .to_owned(),
            )
            .await?;
        manager
            .create_table(
                Table::create()
                    .table(Action::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Action::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Action::Proxy).string())
                    .col(ColumnDef::new(Action::Share).string())
                    .to_owned(),
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // todo!();
        manager
            .drop_table(Table::drop().table(Host::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Match::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Listener::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Action::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
pub enum Host {
    Table, // special attribute
    Id,
    Domain, // Host domain name (ex: "example.com")
}

#[derive(DeriveIden)]
pub enum Listener {
    Table, // special attribute
    Id,
    IpSocket,
    Tls,
}

#[derive(DeriveIden)]
pub enum Match {
    Table, // special attribute
    Id,
    Uri, // Uri path (ex: "/rust-lang/rust/issues")
    Source,
    Category, // Has many hosts [hosts]
}
#[derive(Iden, EnumIter)]
pub enum MatchCategory {
    Managed,
    Unmanaged,
    HttpChallenge,
    TlsAlpnChallenge,
}

#[derive(DeriveIden)]
pub enum Action {
    Table, // special attribute
    Id,
    Proxy,
    Share,
}

#[cfg(test)]
mod tests {
    use crate::{Migrator, MigratorTrait};
    use jucenit_core::{ConfigFile, JuceConfig};
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn create_db() -> Result<()> {
        let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";

        let connection = sea_orm::Database::connect(database_url)
            .await
            .into_diagnostic()?;
        Migrator::up(&connection, None).await.into_diagnostic()?;
        Ok(())
    }

    #[tokio::test]
    async fn seed_db() -> Result<()> {
        // Get struct from config
        let toml = "
            [[unit]]
            listeners = ['*:443']

            [unit.match]
            hosts = ['test.com']

            [unit.action]
            proxy = 'http://127.0.0.1:8333'
        ";
        let config = ConfigFile::from_toml_str(toml)?;
        let unit = config.unit.first().unwrap();

        let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
        let connection = sea_orm::Database::connect(database_url)
            .await
            .into_diagnostic()?;
        Ok(())
    }
}
