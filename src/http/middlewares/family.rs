use crate::application::family::{get_families_from_username};
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::get_username_from_session;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;
use crate::application::errors::ApplicationErrors;

#[derive(Serialize)]
struct FamilyView {
    name: String,
}

pub async fn get_families_middleware<DB, U, F>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    debug!("[Middleware] Getting families");

    let oidc_client = state.oidc_client.clone();
    let username = match oidc_client {
        Some(client) => {
            let client = client.lock().unwrap();
            get_username_from_session(&client, &session).await
        }
        None => None,
    };

    match get_families_from_username(state, username).await {
        Ok(families) => {
            let views = families
                .into_iter()
                .map(|family| FamilyView { name: family.name })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(views)
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
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState};
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};

    #[actix_web::test]
    async fn should_return_unauthorized_without_session() {
        let state = mock_actix_state(Some(vec![
            FamilyEntity {
                id: 1,
                name: "Family A".to_string(),
            },
            FamilyEntity {
                id: 2,
                name: "Family B".to_string(),
            },
        ]));
        let (spy_handler, spy) = spy!();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route(
                    "/families",
                    web::get().to(
                        move |session: actix_session::Session,
                              state: web::Data<MockActixState>| {
                            spy_handler();
                            super::get_families_middleware(session, state)
                        },
                    ),
                ),
        )
            .await;

        let req = test::TestRequest::get().uri("/families").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        drop(app);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}
