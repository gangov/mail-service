use mail_service::configuration::get_configuration;
use mail_service::startup::run;
use sqlx::PgPool;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Can't get config");
    let connection = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Can't connect to Postrgres");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("Can't bind");
    run(listener, connection)?.await
}
