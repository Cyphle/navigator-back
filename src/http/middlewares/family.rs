use crate::application::errors::ApplicationErrors;
use crate::application::family::CreateFamilyCommand;
use crate::config::actix::{ActixState, DbConnection};
use crate::http::requests::family::CreateFamilyRequest;
use crate::repositories::family::FamilyEntity;
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::{get_connected_username, get_username_from_session};
use actix_session::Session;
use actix_web::{HttpResponse, Responder, web};
use log::{debug, error};
use serde::Serialize;
use std::future::Future;

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
    Fut: Future<Output = Result<Vec<FamilyEntity>, ApplicationErrors>>,
{
    debug!("[Middleware] Getting families");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(ApplicationErrors::MissingUsername);

    match username {
        Ok(username) => match get_families(state, username).await {
            Ok(families) => {
                let views = families
                    .into_iter()
                    .map(|family| FamilyView { name: family.name })
                    .collect::<Vec<_>>();
                HttpResponse::Ok().json(views)
            }
            Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
            Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::Conflict().finish(),
            Err(ApplicationErrors::Database(e)) => {
                error!("Error getting families: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
        Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::Conflict().finish(),
        Err(ApplicationErrors::Database(e)) => {
            error!("Error getting families: {:?}", e);
            HttpResponse::InternalServerError().finish()
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
    Fut: Future<Output = Result<crate::domain::family::family::Family, ApplicationErrors>>,
{
    debug!("[Middleware] Creating family middleware");
    let username = get_connected_username(&session, &state)
        .await
        .ok_or(ApplicationErrors::MissingUsername);

    match username {
        Ok(username) => match create_family(
            state,
            username,
            CreateFamilyCommand { name: request.name },
        )
        .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::Conflict().finish(),
            Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
            Err(ApplicationErrors::Database(e)) => {
                error!("Error creating family: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(ApplicationErrors::MissingUsername) => HttpResponse::Unauthorized().finish(),
        Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::Conflict().finish(),
        Err(ApplicationErrors::Database(e)) => {
            error!("Error creating family: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::application::family::{create_family, get_families};
    use crate::http::requests::family::CreateFamilyRequest;
    use crate::repositories::family::FamilyEntity;
    use crate::testing::actix::mock_state::{MockActixState, MockStateConfig, mock_actix_state};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use spy::{Spy, spy};
    use std::sync::Arc;

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
                            get_families(state, username)
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
                                    create_family(state, username, command)
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
