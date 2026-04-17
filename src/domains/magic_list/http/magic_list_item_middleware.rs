use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::magic_list::domain::create_magic_list_item_command::CreateMagicListItemCommand;
use crate::domains::magic_list::domain::magic_list_item_status::MagicListItemStatus;
use crate::domains::magic_list::domain::update_magic_list_item_command::UpdateMagicListItemCommand;
use crate::domains::magic_list::http::magic_list_requests::{CreateMagicListItemRequest, UpdateMagicListItemRequest};
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use chrono::NaiveDate;
use log::{debug, error};
use std::future::Future;

pub async fn add_item_to_magic_list_middleware<DB, CreateItem, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    magic_list_id: i32,
    request: CreateMagicListItemRequest,
    create_item: CreateItem,
) -> impl Responder
where
    DB: DbConnection + Clone,
    CreateItem: Fn(web::Data<ActixState<DB>>, String, i32, CreateMagicListItemCommand) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Adding item to magic list {}", magic_list_id);

    let username = match get_connected_username(&session, &state).await.ok_or(MissingUsernameError) {
        Ok(u) => u,
        Err(e) => {
            error!("Error adding item to magic list: {:?}", e.get_message());
            return HttpResponse::InternalServerError().json(e.get_message());
        }
    };

    let due_date = match request.due_date.as_deref().map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d")) {
        Some(Err(e)) => {
            error!("Invalid due_date format: {}", e);
            return HttpResponse::BadRequest().json("Invalid due_date format, expected YYYY-MM-DD");
        }
        Some(Ok(d)) => Some(d),
        None => None,
    };

    let command = CreateMagicListItemCommand {
        title: request.title,
        content: request.content,
        checked: request.checked,
        due_date,
        status: request.status.as_deref().and_then(MagicListItemStatus::from_str),
    };

    create_item(state, username, magic_list_id, command)
        .await
        .map(|_| HttpResponse::Created().finish())
        .unwrap_or_else(|e| {
            let message = e.get_message();
            match e.status_code() {
                403 => HttpResponse::Forbidden().json(message),
                _ => {
                    error!("Error adding item to magic list: {:?}", message);
                    HttpResponse::InternalServerError().json(message)
                }
            }
        })
}

pub async fn update_item_of_magic_list_middleware<DB, UpdateItem, Fut>(
    session: Session,
    state: web::Data<ActixState<DB>>,
    magic_list_id: i32,
    item_id: i32,
    request: UpdateMagicListItemRequest,
    update_item: UpdateItem,
) -> impl Responder
where
    DB: DbConnection + Clone,
    UpdateItem: Fn(web::Data<ActixState<DB>>, String, i32, i32, UpdateMagicListItemCommand) -> Fut,
    Fut: Future<Output = Result<(), Box<dyn ApplicationError>>>,
{
    debug!("[Middleware] Updating item {} of magic list {}", item_id, magic_list_id);

    let username = match get_connected_username(&session, &state).await.ok_or(MissingUsernameError) {
        Ok(u) => u,
        Err(e) => {
            error!("Error updating item of magic list: {:?}", e.get_message());
            return HttpResponse::InternalServerError().json(e.get_message());
        }
    };

    let due_date = match request.due_date.as_deref().map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d")) {
        Some(Err(e)) => {
            error!("Invalid due_date format: {}", e);
            return HttpResponse::BadRequest().json("Invalid due_date format, expected YYYY-MM-DD");
        }
        Some(Ok(d)) => Some(d),
        None => None,
    };

    let command = UpdateMagicListItemCommand {
        title: request.title,
        content: request.content,
        checked: request.checked,
        due_date,
        status: request.status.as_deref().and_then(MagicListItemStatus::from_str),
    };

    update_item(state, username, magic_list_id, item_id, command)
        .await
        .map(|_| HttpResponse::Ok().finish())
        .unwrap_or_else(|e| {
            let message = e.get_message();
            match e.status_code() {
                403 => HttpResponse::Forbidden().json(message),
                _ => {
                    error!("Error updating item of magic list: {:?}", message);
                    HttpResponse::InternalServerError().json(message)
                }
            }
        })
}

#[cfg(test)]
mod tests {
    use crate::domains::magic_list::http::magic_list_requests::{CreateMagicListItemRequest, UpdateMagicListItemRequest};
    use crate::domains::magic_list::usecases::add_item_to_magic_list_use_case::add_item_to_magic_list_use_case;
    use crate::domains::magic_list::usecases::update_item_of_magic_list_use_case::update_item_of_magic_list_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};
    use std::sync::Arc;

    #[actix_web::test]
    async fn should_call_add_item_to_magic_list_application_layer() {
        // Given
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families/{family_id}/magic-lists/{magic_list_id}/items",
            web::post().to({
                let spy_handler = Arc::clone(&spy_handler);
                move |session: actix_session::Session,
                      state: web::Data<MockActixState>,
                      path: web::Path<(i32, i32)>,
                      request: web::Json<CreateMagicListItemRequest>| {
                    let spy_handler = Arc::clone(&spy_handler);
                    async move {
                        let (_, magic_list_id) = path.into_inner();
                        session
                            .insert("test_username", "mock_user")
                            .expect("failed to set test username in session");
                        super::add_item_to_magic_list_middleware(
                            session,
                            state,
                            magic_list_id,
                            request.into_inner(),
                            move |state, username, magic_list_id, command| {
                                (spy_handler)();
                                add_item_to_magic_list_use_case(state, username, magic_list_id, command)
                            },
                        )
                        .await
                    }
                }
            }),
        ))
        .await;

        let request = CreateMagicListItemRequest {
            title: "Buy milk".to_string(),
            content: None,
            checked: None,
            due_date: None,
            status: None,
        };

        // When
        let req = test::TestRequest::post()
            .uri("/families/1/magic-lists/1/items")
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

    #[actix_web::test]
    async fn should_call_update_item_of_magic_list_application_layer() {
        // Given
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);
        let app = test::init_service(App::new().app_data(state.clone()).route(
            "/families/{family_id}/magic-lists/{magic_list_id}/items/{item_id}",
            web::put().to({
                let spy_handler = Arc::clone(&spy_handler);
                move |session: actix_session::Session,
                      state: web::Data<MockActixState>,
                      path: web::Path<(i32, i32, i32)>,
                      request: web::Json<UpdateMagicListItemRequest>| {
                    let spy_handler = Arc::clone(&spy_handler);
                    async move {
                        let (_, magic_list_id, item_id) = path.into_inner();
                        session
                            .insert("test_username", "mock_user")
                            .expect("failed to set test username in session");
                        super::update_item_of_magic_list_middleware(
                            session,
                            state,
                            magic_list_id,
                            item_id,
                            request.into_inner(),
                            move |state, username, magic_list_id, item_id, command| {
                                (spy_handler)();
                                update_item_of_magic_list_use_case(state, username, magic_list_id, item_id, command)
                            },
                        )
                        .await
                    }
                }
            }),
        ))
        .await;

        let request = UpdateMagicListItemRequest {
            title: Some("Updated".to_string()),
            content: None,
            checked: Some(true),
            due_date: None,
            status: None,
        };

        // When
        let req = test::TestRequest::put()
            .uri("/families/1/magic-lists/1/items/10")
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
