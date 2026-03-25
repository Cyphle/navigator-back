use crate::config::actix::{ActixState, DbConnection};
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::repositories::user_repository::UserRepository;
use crate::domains::user::usecases::get_user_info_use_case::get_user_info_use_case;
use crate::security::token::{get_connected_username, get_username_from_session};
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct UserView {
    pub id: i32,
    username: String,
    // pub email: String,
    // #[serde(rename = "firstName")]
    // pub first_name: String,
    // #[serde(rename = "lastName")]
    // pub last_name: String,
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

    let username = get_connected_username(&session, &state).await;

    if username.is_none() {
        return HttpResponse::Unauthorized().finish();
    }

    match get_user_info_use_case(state, username).await {
        Ok(user) => {
            HttpResponse::Ok().json(UserView {
                id: user.id.unwrap_or(-1),
                username: user.username
            })
        }
        Err(e) => {
            error!("Error getting families: {:?}", e.get_message());
            HttpResponse::InternalServerError().json(e.get_message())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::users_info_middleware;
    use crate::domains::family::repositories::family_entity::FamilyEntity;
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
