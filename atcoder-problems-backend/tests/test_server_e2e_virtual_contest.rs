use actix_web::{http::StatusCode, test, App};
use atcoder_problems_backend::server::{
    config_services,
    middleware::github_auth::{GithubAuthentication, GithubClient, GithubToken},
};
use serde_json::{json, Value};

pub mod utils;

const VALID_CODE: &str = "VALID-CODE";
const VALID_TOKEN: &str = "VALID-TOKEN";

#[actix_web::test]
async fn test_virtual_contest() {
    let mock_server = utils::start_mock_github_server(VALID_TOKEN);
    let mock_server_base_url = mock_server.base_url();
    let mock_api_server = utils::start_mock_github_api_server(VALID_TOKEN, GithubToken { id: 0 });
    let mock_api_server_base_url = mock_api_server.base_url();
    let pg_pool = utils::initialize_and_connect_to_test_sql().await;
    let github =
        GithubClient::new("", "", &mock_server_base_url, &mock_api_server_base_url).unwrap();
    let mut app = test::init_service(
        App::new()
            .wrap(GithubAuthentication::new(github.clone()))
            .app_data(actix_web::web::Data::new(github.clone()))
            .app_data(actix_web::web::Data::new(pg_pool.clone()))
            .configure(config_services),
    )
    .await;

    let request = test::TestRequest::get()
        .uri(&format!("/internal-api/authorize?code={}", VALID_CODE))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::FOUND);

    let cookie_header = format!("token={}", VALID_TOKEN);

    let request = test::TestRequest::post()
        .uri("/internal-api/user/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "atcoder_user_id": "atcoder_user1"
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/create")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "title": "contest title",
            "memo": "contest memo",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 0,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    let contest_id = response["contest_id"].as_str().unwrap();

    let request = test::TestRequest::post()
        .uri("/internal-api/user/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "atcoder_user_id": "atcoder_user1"
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "id": contest_id,
            "title": "contest title",
            "memo": "contest memo",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 300,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/my")
        .append_header(("Cookie", cookie_header.clone()))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": contest_id,
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/joined")
        .append_header(("Cookie", cookie_header.clone()))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response, json!([]));

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/join")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "contest_id": contest_id,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/joined")
        .append_header(("Cookie", cookie_header.clone()))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": contest_id,
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            }
        ])
    );

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/leave")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "contest_id": contest_id,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/joined")
        .append_header(("Cookie", cookie_header.clone()))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response, json!([]));

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/item/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "contest_id": contest_id,
            "problems": [{ "id": "problem_1", "point": 100 }],
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/item/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "contest_id": contest_id,
            "problems": [{ "id": "problem_1", "point": 100 }, { "id": "problem_2" }],
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/join")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "contest_id": contest_id,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri(&format!("/internal-api/contest/get/{}", contest_id))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(
        response,
        json!({
            "info": {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "id": contest_id,
                "mode": null,
                "is_public": true,
                "penalty_second": 300,
            },
            "problems": [{ "id": "problem_1", "point": 100, "order": null }, { "id": "problem_2", "point": null, "order": null }],
            "participants": ["atcoder_user1"],
        })
    );

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/recent")
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(
        response,
        json!([
            {
                "owner_user_id": "0",
                "duration_second": 2,
                "start_epoch_second": 1,
                "memo": "contest memo",
                "title": "contest title",
                "is_public": true,
                "id": contest_id,
                "mode": null,
                "penalty_second": 300,
            }
        ])
    );
}

#[actix_web::test]
async fn test_virtual_contest_visibility() {
    let mock_server = utils::start_mock_github_server(VALID_TOKEN);
    let mock_server_base_url = mock_server.base_url();
    let mock_api_server = utils::start_mock_github_api_server(VALID_TOKEN, GithubToken { id: 0 });
    let mock_api_server_base_url = mock_api_server.base_url();
    let pg_pool = utils::initialize_and_connect_to_test_sql().await;
    let github =
        GithubClient::new("", "", &mock_server_base_url, &mock_api_server_base_url).unwrap();
    let mut app = test::init_service(
        App::new()
            .wrap(GithubAuthentication::new(github.clone()))
            .app_data(actix_web::web::Data::new(github.clone()))
            .app_data(actix_web::web::Data::new(pg_pool.clone()))
            .configure(config_services),
    )
    .await;

    let request = test::TestRequest::get()
        .uri(&format!("/internal-api/authorize?code={}", VALID_CODE))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::FOUND);

    let cookie_header = format!("token={}", VALID_TOKEN);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/create")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "title": "visible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "penalty_second": 300,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    let contest_id = response["contest_id"].as_str().unwrap();

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/recent")
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response[0]["id"].as_str().unwrap(), contest_id);
    assert_eq!(response.as_array().unwrap().len(), 1);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "id": contest_id,
            "title": "invisible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": false,
            "penalty_second": 300,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/recent")
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response.as_array().unwrap().len(), 0);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/create")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "title": "invisible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": false,
            "penalty_second": 300,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    let contest_id = response["contest_id"].as_str().unwrap();

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/recent")
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response.as_array().unwrap().len(), 0);

    let request = test::TestRequest::post()
        .uri("/internal-api/contest/update")
        .append_header(("Cookie", cookie_header.clone()))
        .set_json(json!({
            "id": contest_id,
            "title": "visible",
            "memo": "",
            "start_epoch_second": 1,
            "duration_second": 2,
            "is_public": true,
            "penalty_second": 300,
        }))
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);

    let request = test::TestRequest::get()
        .uri("/internal-api/contest/recent")
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: Value = test::read_body_json(response).await;
    assert_eq!(response.as_array().unwrap().len(), 1);
    assert_eq!(response[0]["id"].as_str().unwrap(), contest_id);
}
