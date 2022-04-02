use std::net::TcpListener;
use sqlx::{Connection, PgConnection};
use mail_service::startup::run;
use mail_service::configuration::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &address))
        .send()
        .await
        .expect("Cant execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_should_return_200_for_valid_form_data() {
    let app_addr = spawn_app();
    let config = get_configuration().expect("Can't get config");
    let connection_string = config.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
      .await
      .expect("Can't connect to Postgres");

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app_addr))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
      .fetch_one(&mut connection)
      .await
      .expect("Can't fetch subscription");

    assert_eq!(saved.email, "ursula_le_guin%40gmail.com");
    assert_eq!(saved.name, "le guin")
}

#[tokio::test]
async fn subscribe_should_throw_error_when_data_is_missing() {
    let app_addr = spawn_app();
    let client = reqwest::Client::new();

    let cases_to_test = vec![
        ("name=le%20guin", "no email"),
        ("no name", "email=ursula_le_guin%40gmail.com"),
        ("no name", "no email"),
    ];

    for (body, err) in cases_to_test {
        let response = client
            .post(&format!("{}/subscriptions", &app_addr))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not fail with 400 Bad Request when the payload was {}.",
            err
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Can't start server");
    let _ = tokio::spawn(server);
    format!("http://127.0.0.1:{}", port)
}
