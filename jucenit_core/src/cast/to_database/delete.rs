// Database
use crate::database::{connect_db, fresh_db};
use crate::nginx::db_into_nginx_conf;
use crate::{ConfigFile, ConfigUnit};
// Sea orm
// use indexmap::IndexMap;
use entity::{prelude::*, *};
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

        // Remove action
        if let Some(action) = &unit.action.clone() {
            let res = Action::find()
                .find_also_related(NgMatch)
                .all(&db)
                .await
                .into_diagnostic()?;
            // println!("{:#?}", res);
            for (action, match_) in res {
                if let Some(match_) = match_ {
                    // If match has one or multiple hosts
                    if let Some(hosts) = &unit.match_.hosts {
                        let hosts_linked = match_
                            .find_related(Host)
                            .filter(Condition::all().add(host::Column::Domain.is_in(hosts)))
                            .all(&db)
                            .await
                            .into_diagnostic()?;
                        for host in hosts_linked {
                            match_host::Entity::delete_many()
                                .filter(
                                    Condition::all()
                                        .add(match_host::Column::HostId.eq(host.id))
                                        .add(match_host::Column::MatchId.eq(match_.id)),
                                )
                                .exec(&db)
                                .await
                                .into_diagnostic()?;
                        }
                    // If match has no hosts
                    } else {
                        let res = Action::find()
                            .find_also_related(NgMatch)
                            .one(&db)
                            .await
                            .into_diagnostic()?;
                        if let Some((action, match_)) = res {
                            if let Some(match_) = match_ {}
                        }
                    }
                }
            }
        }

        // Remove match
        // If unit match apply to specific hosts
        if let Some(hosts) = &unit.match_.hosts {
            let select =
                NgMatch::find().find_with_related(Host).filter(
                    Condition::all()
                        // same hosts
                        .add(host::Column::Domain.is_in(hosts))
                        // and same params
                        .add(ng_match::Column::RawParams.like(
                            serde_json::to_string(&unit.match_.raw_params).into_diagnostic()?,
                        )),
                );
            let res = select.all(&db).await.into_diagnostic()?;
            // println!("{:#?}", res);
            // If host has no other match related
            let matches_: Vec<ng_match::Model> = res.iter().map(|(x, _)| x.to_owned()).collect();
            for (match_, hosts) in res {
                // match_.delete(&db).await.into_diagnostic()?;
                // for host in hosts {
                //     let res = Host::find()
                //         .find_with_related(NgMatch)
                //         .all(&db)
                //         .await
                //         .into_diagnostic()?;
                //     println!("{:#?}", res);
                // }
            }

            // If unit match applies to every hosts (hosts unspecified)
        } else {
            let select = NgMatch::find().filter(
                Condition::all().add(
                    ng_match::Column::RawParams
                        .like(serde_json::to_string(&unit.match_.raw_params).into_diagnostic()?),
                ),
            );
            let ng_matches = select.all(&db).await.into_diagnostic()?;
            // println!("{:#?}", ng_matches);
        }

        // Remove action
        // if let Some(action) = &unit.action {
        //     let action = Action::delete_many()
        //         .filter(
        //             action::Column::RawParams
        //                 .like(serde_json::to_string(&action.raw_params).into_diagnostic()?),
        //         )
        //         .exec(&db)
        //         .await
        //         .into_diagnostic()?;
        // }
        //
        Ok(())
    }
}
