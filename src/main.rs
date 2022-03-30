use std::net::TcpListener;
use mail_service::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
  let listener = TcpListener::bind("127.0.0.1:8000").expect("Can't bind");
  run(listener)?.await
}