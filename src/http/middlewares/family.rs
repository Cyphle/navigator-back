use crate::application::errors::ApplicationErrors;
use crate::application::family::get_families_from_username;
use crate::config::actix::{ActixState, DbConnection};
use crate::http::requests::family::CreateFamilyRequest;
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::{get_connected_username, get_username_from_session};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use log::{debug, error};
use serde::Serialize;

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

    let username = get_connected_username(&session, &state).await;

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

pub async fn create_family_middleware<DB, U, F>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
    request: CreateFamilyRequest,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    debug!("[Middleware] Creating family middleware");
    let username = get_connected_username(&session, &state).await;

    /*
    -> This in application layer
    Get family of user by name
    Check that it does not exist
    if not, create
     */

    HttpResponse::Ok()
}

#[cfg(test)]
mod tests {
    use crate::http::requests::family::CreateFamilyRequest;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{MockActixState, MockStateConfig, mock_actix_state};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use spy::{Spy, spy};

    #[actix_web::test]
    async fn should_call_get_families_application_layer() {
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![
                    FamilyEntity {
                        id: 1,
                        name: "Family A".to_string(),
                    },
                    FamilyEntity {
                        id: 2,
                        name: "Family B".to_string(),
                    },
                ]),
                ..MockStateConfig::default()
            },
        );
        let (spy_handler, spy) = spy!();
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families",
            web::get().to(
                move |session: actix_session::Session, state: web::Data<MockActixState>| {
                    spy_handler();
                    super::get_families_middleware(session, state)
                },
            ),
        ))
        .await;

        let req = test::TestRequest::get().uri("/families").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        drop(app);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }

    #[actix_web::test]
    async fn should_call_create_family_application_layer() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families",
            web::post().to(
                move |session: actix_session::Session,
                      state: web::Data<MockActixState>,
                      request: web::Json<CreateFamilyRequest>| {
                    spy_handler();
                    super::create_family_middleware(session, state, request.into_inner())
                },
            ),
        ))
        .await;
        let request = CreateFamilyRequest {
            name: "My new family".to_string(),
        };

        let req = test::TestRequest::post()
            .uri("/families")
            .set_json(&request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        drop(app);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}
