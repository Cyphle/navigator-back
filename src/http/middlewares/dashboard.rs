use crate::application::errors::ApplicationErrors;
use crate::config::actix::ActixState;
use crate::domain::dashboard::dashboard::Dashboard;
use crate::security::token::get_connected_username;
use actix_session::Session;
use actix_web::{web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;
use std::future::Future;

#[derive(Serialize)]
pub struct DashboardView {
    pub agenda: Vec<String>,
    pub todos: Vec<String>,
    pub weeklyMenu: String,
    pub recipes: Vec<String>,
    pub shopping: String
}

pub async fn get_dashboard_middleware<DB, U, F, GetDashboard, Fut>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
    family_id: String,
    get_dashboard: GetDashboard,
) -> impl Responder
where
    DB: crate::config::actix::DbConnection,
    U: for<'a> crate::repositories::user::UserRepository<<DB as crate::config::actix::DbConnection>::Tx<'a>>,
    F: for<'a> crate::repositories::family::FamilyRepository<<DB as crate::config::actix::DbConnection>::Tx<'a>>,
    GetDashboard: Fn(web::Data<ActixState<DB, U, F>>, String, String) -> Fut,
    Fut: Future<Output = Result<Dashboard, ApplicationErrors>>,
{
    debug!("[Middleware] Getting dashboard");

    let username = get_connected_username(&session, &state)
        .await
        .ok_or(ApplicationErrors::MissingUsername);

    match username {
        Ok(username) => {
            match get_dashboard(state, username, family_id).await {
                Ok(dashboard) => {
                    HttpResponse::Ok().json(DashboardView {
                        agenda: dashboard.agenda,
                        todos: dashboard.todos,
                        weeklyMenu: dashboard.weeklyMenu,
                        recipes: dashboard.recipes,
                        shopping: dashboard.shopping
                    })
                },
                Err(ApplicationErrors::FamilyAlreadyExists) => HttpResponse::BadRequest().json(""),
                Err(ApplicationErrors::Database(_)) => HttpResponse::InternalServerError().json("Database error"),
                Err(ApplicationErrors::MissingUsername) => HttpResponse::InternalServerError().json("No username specified"),
            }
        },
        Err(ApplicationErrors::MissingUsername) => {
            error!("Username not found");
            HttpResponse::InternalServerError().finish()
        },
        Err(ApplicationErrors::FamilyAlreadyExists) | Err(ApplicationErrors::Database(_)) => {
            error!("Error getting dashboard");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::dashboard::dashboard::Dashboard;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use spy::{spy, Spy};
    use std::sync::Arc;

    #[actix_web::test]
    async fn should_call_get_dashboard_from_application_layer() {
        // Given
        let state = mock_actix_state(
            MockPoolPostgres,
            MockStateConfig::default()
        );
        let (spy_handler, spy) = spy!();
        let spy_handler: Arc<dyn Fn() + Send + Sync> = Arc::new(spy_handler);

        // When
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route(
                    "/dashboard/{family_id}",
                    web::get().to({
                        let spy_handler = Arc::clone(&spy_handler);
                        move |session: actix_session::Session, state: web::Data<MockActixState>, path: web::Path<String>| {
                            let spy_handler = Arc::clone(&spy_handler);
                            let family_id = path.into_inner();
                            async move {
                                session
                                    .insert("test_username", "mock_user")
                                    .expect("failed to set test username in session");
                                super::get_dashboard_middleware(session, state, family_id, move |_state, _username, _family_id| {
                                    (spy_handler)();
                                    async {
                                        Ok(Dashboard {
                                            agenda: vec![],
                                            todos: vec![],
                                            weeklyMenu: "".to_string(),
                                            recipes: vec![],
                                            shopping: "".to_string(),
                                        })
                                    }
                                })
                                .await
                            }
                        }
                    })
                )
        ).await;

        let req = test::TestRequest::get().uri("/dashboard/123").to_request();
        let resp = test::call_service(&app, req).await;

        // Then
        assert_eq!(resp.status(), StatusCode::OK);
        drop(app);
        drop(spy_handler);
        let snapshot = spy.snapshot();
        assert_eq!(snapshot.num_of_calls(), 1);
    }
}