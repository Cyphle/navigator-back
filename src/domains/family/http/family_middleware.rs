use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::family::domain::create_family_command::CreateFamilyCommand;
use crate::domains::family::domain::family::Family;
use crate::domains::family::domain::family_role::FamilyRole;
use crate::domains::family::http::family_requests::CreateFamilyRequest;
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::repositories::user_repository::UserRepository;
use crate::security::token::{get_connected_username, get_username_from_session};
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;
use std::future::Future;
use crate::domains::family::repositories::family_entity::FamilyEntity;

#[derive(Serialize)]
struct FamilyView {
    name: String,
}

pub async fn get_families_middleware<DB, U, F, GetFamilies, Fut>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
    get_families: GetFamilies,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
    GetFamilies: Fn(web::Data<ActixState<DB, U, F>>, String) -> Fut,
    Fut: Future<Output = Result<Vec<FamilyEntity>, Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Getting families");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError);

    match username {
        Ok(username) => match get_families(state, username).await {
            Ok(families) => {
                let views = families
                    .into_iter()
                    .map(|family| FamilyView { name: family.name })
                    .collect::<Vec<_>>();
                HttpResponse::Ok().json(views)
            }
            Err(e) => {
                error!("Error getting families: {:?}", e.get_message());
                HttpResponse::InternalServerError().json(e.get_message())
            }
        },
        Err(e) => {
            error!("Error getting families: {:?}", e.get_message());
            HttpResponse::InternalServerError().json(e.get_message())
        }
    }
}

pub async fn create_family_middleware<DB, U, F, CreateFamily, Fut>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
    request: CreateFamilyRequest,
    create_family: CreateFamily,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
    CreateFamily: Fn(web::Data<ActixState<DB, U, F>>, String, CreateFamilyCommand) -> Fut,
    Fut: Future<Output = Result<Family, Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Creating family middleware");
    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError);

    match username {
        Ok(username) => match create_family(
            state,
            username,
            CreateFamilyCommand {
                name: request.name,
                role: FamilyRole::Owner,
            },
        )
        .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                error!("Error getting families: {:?}", e.get_message());
                HttpResponse::InternalServerError().json(e.get_message())
            }
        },
        Err(e) => {
            error!("Error getting families: {:?}", e.get_message());
            HttpResponse::InternalServerError().json(e.get_message())
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domains::family::http::family_requests::CreateFamilyRequest;
    use crate::domains::family::usecases::get_families_use_case::get_families_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};
    use std::sync::Arc;
    use crate::domains::family::repositories::family_entity::FamilyEntity;
    use crate::domains::family::usecases::create_family_use_case::create_family_use_case;

    #[actix_web::test]
    async fn should_call_get_families_application_layer() {
        // Given
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
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);

        // When
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families",
            web::get().to({
                let spy_handler = Arc::clone(&spy_handler);
                move |session: actix_session::Session, state: web::Data<MockActixState>| {
                    let spy_handler = Arc::clone(&spy_handler);
                    async move {
                        session
                            .insert("test_username", "mock_user")
                            .expect("failed to set test username in session");
                        super::get_families_middleware(session, state, move |state, username| {
                            (spy_handler)();
                            get_families_use_case(state, username)
                        })
                        .await
                    }
                }
            }),
        ))
        .await;
        let req = test::TestRequest::get().uri("/families").to_request();
        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), StatusCode::OK);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }

    #[actix_web::test]
    async fn should_call_create_family_application_layer() {
        // Given
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families",
            web::post().to(
                {
                    let spy_handler = Arc::clone(&spy_handler);
                    move |session: actix_session::Session,
                          state: web::Data<MockActixState>,
                          request: web::Json<CreateFamilyRequest>| {
                        let spy_handler = Arc::clone(&spy_handler);
                        async move {
                            session
                                .insert("test_username", "mock_user")
                                .expect("failed to set test username in session");
                            super::create_family_middleware(
                                session,
                                state,
                                request.into_inner(),
                                move |state, username, command| {
                                    (spy_handler)();
                                    create_family_use_case(state, username, command)
                                },
                            )
                            .await
                        }
                    }
                },
            ),
        ))
        .await;
        let request = CreateFamilyRequest {
            name: "My new family".to_string(),
        };

        // When
        let req = test::TestRequest::post()
            .uri("/families")
            .set_json(&request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), StatusCode::OK);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}
