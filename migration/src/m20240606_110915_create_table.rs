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
use sea_query::Index;
use strum::EnumIter;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Junction table Match_Listener
        manager
            .create_table(
                Table::create()
                    .table(MatchListener::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(MatchListener::MatchId)
                            .col(MatchListener::ListenerId),
                    )
                    .col(ColumnDef::new(MatchListener::MatchId).integer().not_null())
                    .col(
                        ColumnDef::new(MatchListener::ListenerId)
                            .integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-match_id")
                            .from(MatchListener::Table, MatchListener::MatchId)
                            .to(NgMatch::Table, NgMatch::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-listener_id")
                            .from(MatchListener::Table, MatchListener::ListenerId)
                            .to(Listener::Table, Listener::Id),
                    )
                    .to_owned(),
            )
            .await?;
        // Junction table Match_Host
        manager
            .create_table(
                Table::create()
                    .table(MatchHost::Table)
                    .if_not_exists()
                    .primary_key(
                        Index::create()
                            .col(MatchHost::MatchId)
                            .col(MatchHost::HostId),
                    )
                    .col(ColumnDef::new(MatchHost::MatchId).integer().not_null())
                    .col(ColumnDef::new(MatchHost::HostId).integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-match_id")
                            .from(MatchHost::Table, MatchHost::MatchId)
                            .to(NgMatch::Table, NgMatch::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-host_id")
                            .from(MatchHost::Table, MatchHost::HostId)
                            .to(Host::Table, Host::Id),
                    )
                    .to_owned(),
            )
            .await?;
        // Match
        manager
            .create_table(
                Table::create()
                    .table(NgMatch::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(NgMatch::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(NgMatch::ActionId).integer().unique_key())
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-action_id")
                            .from(NgMatch::Table, NgMatch::ActionId)
                            .to(Action::Table, Action::Id),
                    )
                    .col(ColumnDef::new(NgMatch::RawParams).json().unique_key())
                    .to_owned(),
            )
            .await?;
        // Listener
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
                    .col(
                        ColumnDef::new(Listener::IpSocket)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(ColumnDef::new(Listener::Tls).json())
                    .to_owned(),
            )
            .await?;
        // Host
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
                    .col(
                        ColumnDef::new(Host::Domain)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;
        // Action
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
                    .col(ColumnDef::new(Action::RawParams).json().unique_key())
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
            .drop_table(Table::drop().table(NgMatch::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Listener::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Action::Table).to_owned())
            .await?;
        // Pivot/Juction tables
        manager
            .drop_table(Table::drop().table(MatchListener::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(MatchHost::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(DeriveIden, Debug)]
pub enum MatchListener {
    Table,
    Id,
    MatchId,
    ListenerId,
}
#[derive(DeriveIden, Debug)]
pub enum MatchHost {
    Table,
    Id,
    MatchId,
    HostId,
}
#[derive(DeriveIden, Debug)]
pub enum Host {
    Table, // special attribute
    Id,
    Domain, // Host domain name (ex: "example.com")
}

#[derive(DeriveIden, Debug)]
pub enum Listener {
    Table, // special attribute
    Id,
    IpSocket,
    Tls,
}

#[derive(DeriveIden, Debug)]
pub enum NgMatch {
    Table, // special attribute
    Id,
    RawParams,
    // Relations
    ActionId,
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
    RawParams,
}

#[cfg(test)]
mod tests {
    use crate::{Migrator, MigratorTrait};
    use miette::{IntoDiagnostic, Result};

    #[tokio::test]
    async fn create_db() -> Result<()> {
        let database_url = "sqlite:////var/spool/jucenit/config.sqlite?mode=rwc";
        let connection = sea_orm::Database::connect(database_url)
            .await
            .into_diagnostic()?;
        Migrator::fresh(&connection).await.into_diagnostic()?;
        Ok(())
    }
}
