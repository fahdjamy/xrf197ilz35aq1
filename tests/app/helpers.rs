use sqlx::{Connection, Executor, PgConnection, PgPool};
use xrf1::configs::DatabaseConfig;

async fn configure_database(config: &DatabaseConfig, db_name: &str) -> PgPool {
    let mut connection = create_pg_connection(&config).await;

    // create a new database based on the db name provided (db_name)
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect_with(config.postgres.connect_to_database(db_name))
        .await
        .expect("Failed to connect to database.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

async fn create_pg_connection(config: &DatabaseConfig) -> PgConnection {
    // Create database: Omitting the db name, we connect to the Postgres instance.
    let connection = PgConnection::connect_with(&config.postgres.connect_to_instance())
        .await
        .expect("Failed to connect to Postgres");

    connection
}

async fn drop_database(mut connection: PgConnection, db_name: &str) {
    connection
        .execute(
            format!(r#"
            DROP DATABASE "{}";
            "#, db_name).as_str()
        )
        .await
        .expect("Failed to drop database.");
}
