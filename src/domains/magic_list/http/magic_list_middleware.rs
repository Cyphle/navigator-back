use crate::config::actix::{ActixState, DbConnection, AsPgConn};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::common::visibility::Visibility;
use crate::domains::magic_list::domain::create_magic_list_command::CreateMagicListCommand;
use crate::domains::magic_list::domain::magic_list_type::MagicListType;
use crate::domains::magic_list::http::magic_list_requests::CreateMagicListRequest;
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use std::future::Future;

pub async fn create_magic_list_middleware<DB, CreateMagicList, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    family_id: i32,
    request: CreateMagicListRequest,
    create_magic_list: CreateMagicList,
) -> impl Responder
where
    DB: DbConnection + Clone + AsPgConn,
    CreateMagicList: Fn(web::Data<ActixState<DB>>, String, CreateMagicListCommand) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Creating magic list");
    
    let username = get_connected_username(&session, &state)
        .await
        .ok_or(MissingUsernameError);

    let username = match username {
        Ok(u) => u,
        Err(e) => {
            error!("Error creating magic list: {:?}", e.get_message());
            return HttpResponse::InternalServerError().json(e.get_message());
        }
    };

    let command = CreateMagicListCommand {
        name: request.name,
        visibility: match request.visibility.to_uppercase().as_str() {
            "PERSONAL" => Visibility::Personal,
            _ => Visibility::Shared,
        },
        magic_list_type: MagicListType::from_str(&request.magic_list_type),
        family_id: Some(family_id),
        excluded_member_ids: request.excluded_member_ids,
    };

    create_magic_list(state, username, command)
        .await
        .map(|_| HttpResponse::Created().finish())
        .unwrap_or_else(|e| {
            error!("Error creating magic list: {:?}", e.get_message());
            HttpResponse::InternalServerError().json(e.get_message())
        })
}

#[cfg(test)]
mod tests {
    use crate::domains::magic_list::http::magic_list_requests::CreateMagicListRequest;
    use crate::domains::magic_list::usecases::create_magic_list_use_case::create_magic_list_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};
    use std::sync::Arc;

    #[actix_web::test]
    async fn should_call_create_magic_list_application_layer() {
        // Given
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families/{family_id}/magic-lists",
            web::post().to(
                {
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
                                move |state, username, command| {
                                    (spy_handler)();
                                    create_magic_list_use_case(state, username, command)
                                },
                            )
                            .await
                        }
                    }
                },
            ),
        ))
        .await;

        let request = CreateMagicListRequest {
            name: "My list".to_string(),
            visibility: "SHARED".to_string(),
            magic_list_type: "TASK".to_string(),
            family_id: None,
            excluded_member_ids: None,
        };

        // When
        let req = test::TestRequest::post()
            .uri("/families/1/magic-lists")
            .set_json(&request)
            .to_request();
        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), StatusCode::CREATED);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}
