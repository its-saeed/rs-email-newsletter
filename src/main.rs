use sqlx::PgPool;
use std::net::TcpListener;
use zero_2_prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Unable to connect to database");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;
    run(listener, connection_pool)?.await
}
