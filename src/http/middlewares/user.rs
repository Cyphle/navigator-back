use crate::application::errors::ApplicationErrors;
use crate::application::user::get_users_me;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::get_username_from_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;

#[derive(Serialize)]
struct UserView {
    username: String,
}

pub async fn users_me_middleware<DB, U, F>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    debug!("Calling users me");

    let oidc_client = state.oidc_client.clone();
    let username = match oidc_client {
        Some(client) => {
            let client = client.lock().unwrap();
            get_username_from_session(&client, &session).await
        }
        None => None,
    };

    match get_users_me(state, username).await {
        Ok(user) => {
            HttpResponse::Ok().json(UserView { username: user.username })
        }
        Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
        Err(ApplicationErrors::Database(e)) => {
            error!("Error getting families: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::users_me_middleware;
    use crate::config::actix::ActixState;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use crate::testing::repositories::mock_family_repository::MockFamilyRepository;
    use crate::testing::repositories::mock_user_repository::MockUserRepository;
    use crate::testing::security::oidc::dummy_oidc_config;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use openid::{Client, Config, Discovered, StandardClaims};
    use std::sync::{Arc, Mutex};
    use url::Url;

    fn dummy_oidc_client() -> Arc<Mutex<Client<Discovered, StandardClaims>>> {
        let base = Url::parse("https://example.com").unwrap();
        let config = Config {
            issuer: base.clone(),
            authorization_endpoint: base.join("/authorize").unwrap(),
            token_endpoint: base.join("/token").unwrap(),
            userinfo_endpoint: Some(base.join("/userinfo").unwrap()),
            jwks_uri: base.join("/jwks").unwrap(),
            registration_endpoint: None,
            scopes_supported: Some(vec!["openid".to_string()]),
            response_types_supported: vec!["code".to_string()],
            response_modes_supported: None,
            grant_types_supported: None,
            acr_values_supported: None,
            subject_types_supported: vec!["public".to_string()],
            id_token_signing_alg_values_supported: vec!["RS256".to_string()],
            id_token_encryption_alg_values_supported: None,
            id_token_encryption_enc_values_supported: None,
            userinfo_signing_alg_values_supported: None,
            userinfo_encryption_alg_values_supported: None,
            userinfo_encryption_enc_values_supported: None,
            request_object_signing_alg_values_supported: None,
            request_object_encryption_alg_values_supported: None,
            request_object_encryption_enc_values_supported: None,
            token_endpoint_auth_methods_supported: None,
            token_endpoint_auth_signing_alg_values_supported: None,
            display_values_supported: None,
            claim_types_supported: None,
            claims_supported: None,
            service_documentation: None,
            claims_locales_supported: None,
            ui_locales_supported: None,
            claims_parameter_supported: false,
            request_parameter_supported: false,
            request_uri_parameter_supported: true,
            require_request_uri_registration: false,
            op_policy_uri: None,
            op_tos_uri: None,
            end_session_endpoint: None,
            introspection_endpoint: None,
            code_challenge_methods_supported: None,
        };
        let http_client = reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap();
        let client = Client::new(
            Discovered::from(config),
            "client".to_string(),
            Some("secret".to_string()),
            Some(base.join("/redirect").unwrap().to_string()),
            http_client,
            None,
        );
        Arc::new(Mutex::new(client))
    }

    fn make_state(
    ) -> web::Data<ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockPoolPostgres,
            oidc_config: dummy_oidc_config(),
            oidc_client: Some(dummy_oidc_client()),
            user_repository: Arc::new(MockUserRepository::default()),
            family_repository: Arc::new(MockFamilyRepository {
                families: vec![FamilyEntity {
                    id: 1,
                    name: "Family A".to_string(),
                }],
            }),
        })
    }

    #[actix_web::test]
    async fn should_return_ok_without_bearer() {
        let state = make_state();
        let app = test::init_service(
            App::new().app_data(state.clone()).route(
                "/users/me",
                web::get().to(
                    move |session: actix_session::Session,
                          state: web::Data<
                              ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>,
                          >| { users_me_middleware(session, state) },
                ),
            ),
        )
        .await;

        let req = test::TestRequest::get().uri("/users/me").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
