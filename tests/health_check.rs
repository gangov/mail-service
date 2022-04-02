use mail_service::configuration::{get_configuration, DatabaseSettings};
use mail_service::startup::run;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub pool: PgPool,
}

#[tokio::test]
async fn health_check_works() {
    let app_addr = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", &app_addr.address))
        .send()
        .await
        .expect("Cant execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_should_return_200_for_valid_form_data() {
    let app_addr = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let response = client
        .post(&format!("{}/subscriptions", &app_addr.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app_addr.pool)
        .await
        .expect("Can't fetch subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin")
}

#[tokio::test]
async fn subscribe_should_throw_error_when_data_is_missing() {
    let app_addr = spawn_app().await;
    let client = reqwest::Client::new();

    let cases_to_test = vec![
        ("name=le guin", "no email"),
        ("no name", "email=ursula_le_guin%@gmail.com"),
        ("no name", "no email"),
    ];

    for (body, err) in cases_to_test {
        let response = client
            .post(&format!("{}/subscriptions", &app_addr.address))
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

async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to port");
    let port = listener.local_addr().unwrap().port();
    let addr = format!("http://127.0.0.1:{}", port);
    let mut configuration = get_configuration().expect("Can't read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let connection = configure_db(&configuration.database).await;

    let server = run(listener, connection.clone()).expect("Can't start server");
    let _ = tokio::spawn(server);
    TestApp {
        address: addr,
        pool: connection,
    }
}

pub async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_no_db())
        .await
        .expect("Can't connect to db");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Can't create new database");

    let connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Can't connect to Postgres");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Cant do the migrations");

    connection_pool
}
