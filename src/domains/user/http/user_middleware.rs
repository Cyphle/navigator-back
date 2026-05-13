use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::middleware_error::MiddlewareError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::user::http::user_views::UserView;
use crate::domains::user::usecases::get_user_info_use_case::get_user_info_use_case;
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use log::debug;

pub async fn users_info_middleware<DB: DbConnection>(
    session: Session,
    state: web::Data<ActixState<DB>>,
) -> Result<HttpResponse, MiddlewareError>
where
    for<'a> <DB as DbConnection>::Tx<'a>: AsPgConn,
{
    debug!("[Middleware] users me");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError)?;

    let user = get_user_info_use_case(state, username).await?;

    Ok(HttpResponse::Ok().json(UserView {
        id: user.id,
        username: user.username,
        email: user.email,
        first_name: user.first_name,
        last_name: user.last_name,
    }))
}

#[cfg(test)]
mod tests {
    use super::users_info_middleware;
    use crate::domains::family::domain::family::Family;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};

    #[actix_web::test]
    async fn should_return_unauthorized_without_session() {
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![Family {
                    id: 1,
                    name: "Family A".to_string(),
                    creator_username: "johndoe".to_string(),
                    members: vec![],
                    active: true
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
