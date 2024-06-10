pub use sea_orm_migration::prelude::*;

mod m20240606_110915_create_table;

pub use m20240606_110915_create_table::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20240606_110915_create_table::Migration)]
    }
}
