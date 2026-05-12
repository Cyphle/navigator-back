use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::middleware_error::MiddlewareError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::family::domain::family::Family;
use crate::domains::family::http::family_requests::CreateFamilyRequest;
use crate::domains::family::usecases::create_family_use_case::CreateFamilyMemberInput;
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{web, HttpResponse};
use log::debug;
use serde::Serialize;
use std::future::Future;

#[derive(Serialize)]
struct FamilyView {
    name: String,
}

pub async fn get_families_middleware<DB, GetFamilies, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    get_families: GetFamilies,
) -> Result<HttpResponse, MiddlewareError>
where
    DB: DbConnection,
    GetFamilies: Fn(web::Data<ActixState<DB>>, String) -> Fut,
    Fut: Future<Output = Result<Vec<Family>, Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Getting families");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError)?;

    let families = get_families(state, username).await?;
    let views = families
        .into_iter()
        .map(|family| FamilyView { name: family.name })
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(views))
}

pub async fn create_family_middleware<DB, CreateFamily, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    request: CreateFamilyRequest,
    create_family: CreateFamily,
) -> Result<HttpResponse, MiddlewareError>
where
    DB: DbConnection,
    CreateFamily: Fn(
        web::Data<ActixState<DB>>,
        String,
        String,
        String,
        Vec<CreateFamilyMemberInput>,
    ) -> Fut,
    Fut: Future<Output = Result<Family, Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Creating family middleware");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError)?;

    let members = request
        .members
        .into_iter()
        .map(|m| CreateFamilyMemberInput {
            username_or_email: m.username_or_email,
            relation: m.relation,
            is_admin: m.is_admin,
        })
        .collect();

    create_family(state, username, request.name, request.creator_relation, members).await?;
    Ok(HttpResponse::Ok().finish())
}

#[cfg(test)]
mod tests {
    use crate::domains::family::http::family_requests::{CreateFamilyMemberRequest, CreateFamilyRequest};
    use crate::domains::family::repositories::family_entity::FamilyEntity;
    use crate::domains::family::usecases::create_family_use_case::create_family_use_case;
    use crate::domains::family::usecases::get_families_use_case::get_families_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::http::mock_requests::MockFamilyRequest;
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};
    use std::sync::Arc;
    use crate::domains::family::domain::family::Family;

    #[actix_web::test]
    async fn should_call_get_families_application_layer() {
        // Given
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                families: Some(vec![
                    Family {
                        id: 1,
                        name: "Family A".to_string(),
                        active: true,
                        creator_username: "johndoe".to_string(),
                        members: vec![],
                    },
                    Family {
                        id: 2,
                        name: "Family B".to_string(),
                        active: true,
                        creator_username: "johnsmith".to_string(),
                        members: vec![],
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
                                .expect("failed to set test username_or_email in session");
                            super::create_family_middleware(
                                session,
                                state,
                                request.into_inner(),
                                move |state, username, name, creator_relation, members| {
                                    (spy_handler)();
                                    create_family_use_case(
                                        state,
                                        username,
                                        name,
                                        creator_relation,
                                        members,
                                    )
                                },
                            )
                            .await
                        }
                    }
                },
            ),
        ))
        .await;

        let request = MockFamilyRequest::new("My new family".to_string())
            .add_creator_relation("PARENT".to_string())
            .add_member(
                CreateFamilyMemberRequest {
                    username_or_email: "mock_user".to_string(),
                    relation: "PARENT".to_string(),
                    is_admin: false
                }
            )
            .build();

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
