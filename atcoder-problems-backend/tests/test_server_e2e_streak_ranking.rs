use rand::Rng;
use serde_json::{json, Value};
use sql_client::PgPool;

pub mod utils;

async fn prepare_data_set(conn: &PgPool) {
    sql_client::query(
        r"INSERT INTO max_streaks (user_id, streak) VALUES ('u1', 1), ('u2', 2), ('u3', 1)",
    )
    .execute(conn)
    .await
    .unwrap();
}

fn url(path: &str, port: u16) -> String {
    format!("http://localhost:{}{}", port, path)
}

async fn setup() -> u16 {
    prepare_data_set(&utils::initialize_and_connect_to_test_sql().await).await;
    let mut rng = rand::thread_rng();
    rng.gen::<u16>() % 30000 + 30000
}

#[actix_web::test]
async fn test_streak_ranking() {
    let port = setup().await;
    let server = actix_web::rt::spawn(async move {
        let pg_pool = sql_client::initialize_pool(utils::get_sql_url_from_env())
            .await
            .unwrap();
        actix_web::HttpServer::new(move || {
            actix_web::App::new()
                .app_data(actix_web::web::Data::new(pg_pool.clone()))
                .configure(atcoder_problems_backend::server::config_services)
        })
        .bind(("0.0.0.0", port))
        .unwrap()
        .run()
        .await
        .unwrap();
    });
    actix_web::rt::time::sleep(std::time::Duration::from_millis(1000)).await;

    // get_streak_ranking(from..to)

    let response = reqwest::get(url("/atcoder-api/v3/streak_ranking?from=0&to=10", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {"user_id": "u2", "count": 2},
            {"user_id": "u1", "count": 1},
            {"user_id": "u3", "count": 1}
        ])
    );

    let response = reqwest::get(url("/atcoder-api/v3/streak_ranking?from=1&to=3", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(
        response,
        json!([
            {"user_id": "u1", "count": 1},
            {"user_id": "u3", "count": 1}
        ])
    );

    let response = reqwest::get(url("/atcoder-api/v3/streak_ranking?from=10&to=0", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response.as_array().unwrap().len(), 0);

    let response = reqwest::get(url("/atcoder-api/v3/streak_ranking?from=0&to=2000", port))
        .await
        .unwrap();
    assert_eq!(response.status(), 400);

    let response = reqwest::get(url("/atcoder-api/v3/streak_ranking?from=-1&to=10", port))
        .await
        .unwrap();
    assert_eq!(response.status(), 400);

    // get_users_streak_rank(user_id)

    let response = reqwest::get(url("/atcoder-api/v3/user/streak_rank?user=u1", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response, json!({"count":1,"rank":1}));

    let response = reqwest::get(url("/atcoder-api/v3/user/streak_rank?user=u2", port))
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();
    assert_eq!(response, json!({"count":2,"rank":0}));

    let response = reqwest::get(url(
        "/atcoder-api/v3/user/streak_rank?user=does_not_exist",
        port,
    ))
    .await
    .unwrap();
    assert_eq!(response.status(), 404);

    let response = reqwest::get(url("/atcoder-api/v3/user/streak_rank?bad=request", port))
        .await
        .unwrap();
    assert_eq!(response.status(), 400);

    server.abort();
    server.await.unwrap_err();
}
