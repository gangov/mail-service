use mail_service::configuration::get_configuration;
use mail_service::startup::run;
use mail_service::telemetry::{get_subscriber, init_subsciber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("mail-service".into(), "info".into(), std::io::stdout);
    init_subsciber(subscriber);

    let configuration = get_configuration().expect("Can't get config");
    let connection = PgPool::connect(configuration.database.connection_string().expose_secret())
        .await
        .expect("Can't connect to Postrgres");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("Can't bind");
    run(listener, connection)?.await
}
