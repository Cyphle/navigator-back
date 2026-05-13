use crate::config::actix::{ActixState, AsPgConn, DbConnection};
use crate::domains::common::errors::middleware_error::MiddlewareError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::magic_list::domain::errors::{CreateMagicListError, GetMagicListSummaryError};
use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use crate::domains::magic_list::http::magic_list_requests::CreateMagicListRequest;
use crate::domains::magic_list::http::magic_list_views::MagicListSummaryView;
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{HttpResponse, web};
use log::debug;
use std::future::Future;

pub async fn get_magic_list_summary_middleware<DB, GetSummary, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    family_id: i32,
    get_summary: GetSummary,
) -> Result<HttpResponse, MiddlewareError>
where
    DB: DbConnection + Clone,
    GetSummary: Fn(web::Data<ActixState<DB>>, String, i32) -> Fut,
    Fut: Future<Output = Result<Vec<MagicListSummary>, GetMagicListSummaryError>>,
{
    debug!(
        "[Middleware] Getting magic list summary for family {}",
        family_id
    );

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError)?;

    let summaries = get_summary(state, username, family_id).await?;
    let views: Vec<MagicListSummaryView> = summaries
        .into_iter()
        .map(MagicListSummaryView::from)
        .collect();
    Ok(HttpResponse::Ok().json(views))
}

pub async fn create_magic_list_middleware<DB, CreateMagicList, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    family_id: i32,
    request: CreateMagicListRequest,
    create_magic_list: CreateMagicList,
) -> Result<HttpResponse, MiddlewareError>
where
    DB: DbConnection + Clone + AsPgConn,
    CreateMagicList: Fn(
        web::Data<ActixState<DB>>,
        String,
        String,
        String,
        String,
        Option<i32>,
        Option<Vec<i32>>,
    ) -> Fut,
    Fut: Future<Output = Result<(), CreateMagicListError>>,
{
    debug!("[Middleware] Creating magic list");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError)?;

    create_magic_list(
        state,
        username,
        request.name,
        request.visibility,
        request.magic_list_type,
        Some(family_id),
        request.excluded_member_ids,
    )
    .await?;

    Ok(HttpResponse::Created().finish())
}

#[cfg(test)]
mod tests {
    use crate::domains::common::visibility::Visibility;
    use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
    use crate::domains::magic_list::domain::magic_list_type::MagicListType;
    use crate::domains::magic_list::http::magic_list_requests::CreateMagicListRequest;
    use crate::domains::magic_list::usecases::create_magic_list_use_case::create_magic_list_use_case;
    use crate::domains::magic_list::usecases::get_magic_list_summary_use_case::get_magic_list_summary_use_case;
    use crate::testing::actix::mock_state::{
        MockActixState, MockMagicListConfig, MockStateConfig, mock_actix_state,
    };
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{App, test, web};
    use spy::{Spy, spy};
    use std::sync::Arc;

    #[actix_web::test]
    async fn should_call_get_magic_list_summary_application_layer() {
        let summaries = vec![MagicListSummary {
            id: 1,
            name: "Courses".to_string(),
            visibility: Visibility::Shared,
            magic_list_type: MagicListType::Simple,
            family_id: Some(1),
            item_count: 3,
        }];
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                magic_list: MockMagicListConfig {
                    summaries,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families/{family_id}/magic-lists/summary",
            web::get().to({
                let spy_handler = Arc::clone(&spy_handler);
                move |session: actix_session::Session,
                      state: web::Data<MockActixState>,
                      family_id: web::Path<i32>| {
                    let spy_handler = Arc::clone(&spy_handler);
                    async move {
                        session
                            .insert("test_username", "mock_user")
                            .expect("failed to set test username in session");
                        super::get_magic_list_summary_middleware(
                            session,
                            state,
                            family_id.into_inner(),
                            move |state, username, family_id| {
                                (spy_handler)();
                                get_magic_list_summary_use_case(state, username, family_id)
                            },
                        )
                        .await
                    }
                }
            }),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/families/1/magic-lists/summary")
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::OK);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }

    #[actix_web::test]
    async fn should_call_create_magic_list_application_layer() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families/{family_id}/magic-lists",
            web::post().to({
                let spy_handler = Arc::clone(&spy_handler);
                move |session: actix_session::Session,
                      state: web::Data<MockActixState>,
                      family_id: web::Path<i32>,
                      request: web::Json<CreateMagicListRequest>| {
                    let spy_handler = Arc::clone(&spy_handler);
                    async move {
                        session
                            .insert("test_username", "mock_user")
                            .expect("failed to set test username in session");
                        super::create_magic_list_middleware(
                            session,
                            state,
                            family_id.into_inner(),
                            request.into_inner(),
                            move |state, username, name, visibility, magic_list_type, family_id, excluded_member_ids| {
                                (spy_handler)();
                                create_magic_list_use_case(
                                    state,
                                    username,
                                    name,
                                    visibility,
                                    magic_list_type,
                                    family_id,
                                    excluded_member_ids,
                                )
                            },
                        )
                        .await
                    }
                }
            }),
        ))
        .await;

        let request = CreateMagicListRequest {
            name: "My list".to_string(),
            visibility: "SHARED".to_string(),
            magic_list_type: "TASK".to_string(),
            family_id: None,
            excluded_member_ids: None,
        };

        let req = test::TestRequest::post()
            .uri("/families/1/magic-lists")
            .set_json(&request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), StatusCode::CREATED);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}
