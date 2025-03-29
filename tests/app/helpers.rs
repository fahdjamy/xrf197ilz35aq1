use sqlx::{Acquire, ConnectOptions, Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use xrf1::configs::{load_config, DatabaseConfig};

#[derive(Debug, Clone)]
pub struct TestApp {
    db_name: String,
    pub user_fp: String,
    pub db_pool: PgPool,
    db_config: DatabaseConfig,
}

impl TestApp {
    pub async fn drop_db(self) {
        // Shut down the connection pool, immediately waking all tasks waiting for a connection.
        // Upon calling this method, any currently waiting or subsequent calls to Pool::acquire and 
        // the like will immediately return Error::PoolClosed and no new connections will be opened.
        // Checked-out connections are unaffected, but will be gracefully closed on-drop rather
        // than being returned to the pool.
        self.db_pool.close().await;

        // 2. Connect to the base PostgresSQL instance again to drop the created DB
        //    It's crucial that no connections remain to the target DB.
        let connection = self.db_config.postgres.connect_to_instance()
            .connect()
            .await
            .expect("could not connect to database");
        drop_database(connection, &self.db_name).await;
    }
}

pub async fn start_test_app() -> TestApp {
    let configs = load_config().expect("Failed to load config");

    // create the PgConnection and wrap it with Arc<Mutex<>>
    let mut connection = create_pg_connection(&configs.database).await;

    let db_name = Uuid::new_v4().to_string();
    let pg_pool = configure_database(&mut connection, &configs.database, &db_name).await;

    TestApp {
        db_name,
        db_pool: pg_pool,
        db_config: configs.database,
        user_fp: Uuid::new_v4().to_string(),
    }
}

pub async fn configure_database(conn: &mut PgConnection, config: &DatabaseConfig, db_name: &str) -> PgPool {
    conn
        .acquire()
        .await
        .expect("Failed to acquire a connection to the database")
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
        .await
        .expect("Failed to create database.");

    // create a new database based on the db name provided (db_name)
    let connection_pool = PgPool::connect_with(config.postgres.connect_to_database(db_name))
        .await
        .expect("Failed to connect to database.");

    // Migrate database
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

async fn drop_database(mut conn: PgConnection, db_name: &str) {
    // Forcefully terminate any remaining connections to the target database.
    // pg_terminate_backend(pid) is a PostgresSQL function that terminates the backend process w/ the given process ID (pid)
    // Because of cases (e.g., bugs in sqlx, race conditions, or issues w/ the PostgresSQL setup)
    // where connections don't get closed immediately
    conn.execute(
        // All connections to the DB should be dropped except for the "pg_conn" connection's PID
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

    conn.execute(
        format!(
            r#"
            DROP DATABASE "{}";
            "#,
            db_name
        )
            .as_str(),
    )
        .await
        .expect("Failed to drop database.");
}
