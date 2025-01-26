use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use xrf1::configs::{load_config, DatabaseConfig};

pub struct TestApp {
    db_name: String,
    pub db_pool: PgPool,
    // Arc: Allow shared ownership of the pg_conn across multiple threads
    // Mutex: Provide exclusive access to pg_conn. Only 1 task can hold the lock & access/mutate the pg_conn at a time
    pg_conn: Arc<Mutex<PgConnection>>,
}

impl TestApp {
    pub async fn drop_db(&self) {
        drop_database(self.pg_conn.clone(), &self.db_name).await;
    }
}

pub async fn start_test_app() -> TestApp {
    let configs = load_config()
        .expect("Failed to load config");

    // create the PgConnection and wrap it with Arc<Mutex<>>
    let pg_conn = Arc::new(Mutex::new(create_pg_connection(&configs.database).await));

    let db_name = Uuid::new_v4().to_string();
    let pg_pool = configure_database(pg_conn.clone(), &configs.database, &db_name)
        .await;

    TestApp {
        db_name,
        pg_conn,
        db_pool: pg_pool,
    }
}

pub async fn configure_database(connection: Arc<Mutex<PgConnection>>,
                                config: &DatabaseConfig,
                                db_name: &str) -> PgPool {

    // create a new database based on the db name provided (db_name)
    connection
        .lock()
        .expect("Failed to acquire lock connection to database instance :: create-database")
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

async fn drop_database(connection: Arc<Mutex<PgConnection>>, db_name: &str) {
    connection
        .lock()
        .expect("Failed to acquire lock connection to database instance :: drop-database")
        .execute(
            format!(r#"
            DROP DATABASE "{}";
            "#, db_name).as_str()
        )
        .await
        .expect("Failed to drop database.");
}
