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
    pub async fn drop_db(self) {
        // MUST drop db_pool before you drop the database
        // self.db_pool (a sqlx::PgPool) holds a pool of active connections to PostgresSQL DB
        // in our case except for the pg_conn's connection to the DB
        // If you try to drop the DB while these connections (db_pool) are still active,
        // PostgresSQL throws the error "database is being accessed by other users."
        // Explicitly calling drop(self.db_pool) will force the pool to close its connections
        // before we attempt to drop the DB, ensuring that the DB is no longer in use by the app
        // For sqlx::PgPool, the drop implementation closes all the connections in the pool
        drop(self.db_pool);
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
        .lock() // calling .lock() returns back a MutexGuard
        // PgConnection is accessed via a MutexGuard w/c implement Deref & DerefMut so you can use
        // it almost as if you had a direct reference to the underlying data
        // when the MutexGuard goes out of scope & is dropped, the lock on the Mutex is automatically released
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
    let mut conn = connection
        .lock()
        .expect("Failed to acquire lock connection to database instance :: drop-database");

    // Forcefully terminate any remaining connections to the target database.
    // pg_terminate_backend(pid) is a PostgresSQL function that terminates the backend process w/ the given process ID (pid)
    // most cases, the drop(self.db_pool),will Drop all pool connections. However, this acts as a 
    // safety net. In cases (e.g., bugs in sqlx, race conditions, or issues w/ the PostgresSQL setup)
    // where connections don't get closed immediately
    conn
        .execute(
            // The pg_conn in TestApp holds a separate connection specifically for creating and dropping the DB
            // all connections to the DB should be dropped except for the "pg_conn" connection's PID
            format!(
                r#"
                SELECT pg_terminate_backend(pg_stat_activity.pid)
                FROM pg_stat_activity
                WHERE pg_stat_activity.datname = '{}' AND pid <> pg_backend_pid();
                "#,
                db_name
            )
                .as_str(),
        )
        .await
        .expect("Failed to terminate other connections");

    conn
        .execute(
            format!(r#"
            DROP DATABASE "{}";
            "#, db_name).as_str()
        )
        .await
        .expect("Failed to drop database.");
}
