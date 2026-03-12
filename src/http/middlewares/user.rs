use crate::application::errors::ApplicationErrors;
use crate::application::user::get_users_info;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::get_username_from_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;

#[derive(Serialize)]
pub struct UserView {
    username: String,
}

pub async fn users_info_middleware<DB, U, F>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    debug!("[Middleware] users me");

    let oidc_client = state.oidc_client.clone();
    let username = match oidc_client {
        Some(client) => {
            let client = client.lock().unwrap();
            get_username_from_session(&client, &session).await
        }
        None => None,
    };

    match get_users_info(state, username).await {
        Ok(user) => {
            HttpResponse::Ok().json(UserView { username: user.username })
        }
        Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
        Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::Conflict().finish(),
        Err(ApplicationErrors::Database(e)) => {
            error!("Error while getting user info: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::users_info_middleware;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn should_return_unauthorized_without_session() {
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![FamilyEntity {
                    id: 1,
                    name: "Family A".to_string(),
                }]),
                ..MockStateConfig::default()
            }
        );
        let app = test::init_service(
            App::new().app_data(state.clone()).route(
                "/users/me",
                web::get().to(
                    move |session: actix_session::Session,
                          state: web::Data<MockActixState>| {
                        users_info_middleware(session, state)
                    },
                ),
            ),
        )
        .await;

        let req = test::TestRequest::get().uri("/users/me").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
