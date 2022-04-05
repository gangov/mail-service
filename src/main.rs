use mail_service::configuration::get_configuration;
use mail_service::startup::run;
use mail_service::telemetry::{get_subscriber, init_subsciber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("mail-service".into(), "info".into(), std::io::stdout);
    init_subsciber(subscriber);

    let configuration = get_configuration().expect("Can't get config");
    let connection = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let addr = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = TcpListener::bind(addr).expect("Can't bind");
    run(listener, connection)?.await
}
