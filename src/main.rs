mod schema;

use diesel::migration::MigrationVersion;
use diesel::pg::{Pg, PgConnection};
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::error::Error;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn main() {
    let connection = establish_connection();
    if let Err(e) = connection {
        eprintln!("Failed to connect to the database: {}", e);
        return;
    }
    let mut connection = connection.unwrap();

    let migrations = run_migrations(&mut connection);
    if let Err(e) = migrations {
        eprintln!("Failed to run migrations: {}", e);
        return;
    }
    let migrations = migrations.unwrap();
    for version in migrations {
        println!("Executed migration {}", version);
    }
}

fn establish_connection() -> ConnectionResult<PgConnection> {
    // TODO: Replace by configuration
    let database_url = "postgres://rosary_music:rosary_music@database/rosary_music_db";
    PgConnection::establish(database_url)
}

fn run_migrations(
    connection: &mut impl MigrationHarness<Pg>,
) -> Result<Vec<MigrationVersion>, Box<dyn Error + Send + Sync + 'static>> {
    connection.run_pending_migrations(MIGRATIONS)
}
