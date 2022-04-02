use std::net::TcpListener;
use mail_service::startup::run;
use mail_service::configuration::get_configuration;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Can't get config");
    let addr = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(addr).expect("Can't bind");
    run(listener)?.await
}
