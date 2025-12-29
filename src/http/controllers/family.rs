use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::get_username_from_session;

#[derive(Serialize)]
struct FamilyView {
    name: String,
}

#[get("/families")]
pub async fn get_families(session: Session, state: web::Data<ActixState>) -> impl Responder {
    debug!("Getting families");

    let oidc_client = state.oidc_client.clone();
    let username = match oidc_client {
        Some(client) => {
            let client = client.lock().unwrap();
            get_username_from_session(&client, &session).await
        }
        None => None,
    };

    get_families_from_username(state, username).await
}

async fn get_families_from_username<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: Option<String>,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    let username = match username {
        Some(username) => username,
        None => return HttpResponse::Unauthorized().finish(),
    };

    let mut tx = match state.db_connection.begin().await {
        Ok(tx) => tx,
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string()),
    };

    match state
        .family_repository
        .get_family_by_member_username(&mut tx, &username)
        .await
    {
        Ok(families) => {
            let views = families
                .into_iter()
                .map(|family| FamilyView { name: family.name })
                .collect::<Vec<_>>();
            HttpResponse::Ok().json(views)
        }
        Err(e) => {
            error!("Error getting families: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::get_families_from_username;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App};
    use async_trait::async_trait;
    use std::pin::Pin;
    use std::sync::Arc;
    use crate::config::actix::{ActixState, DbConnection};
    use crate::domain::user::user::User;
    use crate::repositories::family::{FamilyEntity, FamilyRepository};
    use crate::repositories::user::UserRepository;
    use crate::security::oidc::{OidcAdminConfig, OidcClientConfig, OidcConfig};
    use crate::testing::security::oidc::dummy_oidc_config;

    struct MockPoolPostgres;

    struct MockTransaction;

    impl DbConnection for MockPoolPostgres {
        type Tx<'a> = MockTransaction;

        fn begin<'a>(
            &'a self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
            Box::pin(async { Ok(MockTransaction) })
        }
    }

    struct MockUserRepository;

    #[async_trait]
    impl UserRepository<MockTransaction> for MockUserRepository {
        async fn create_user(
            &self,
            _tx: &mut MockTransaction,
            _user: &User,
        ) -> Result<(u64, actix_web::http::StatusCode), sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn get_user(
            &self,
            _tx: &mut MockTransaction,
            _username: &str,
        ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }

        async fn get_or_create_user(
            &self,
            _tx: &mut MockTransaction,
            _user: &User,
        ) -> Result<crate::repositories::user::UserEntity, sqlx::Error> {
            Err(sqlx::Error::RowNotFound)
        }
    }

    struct MockFamilyRepository {
        families: Vec<FamilyEntity>,
    }

    #[async_trait]
    impl FamilyRepository<MockTransaction> for MockFamilyRepository {
        async fn get_family_by_member_username(
            &self,
            _tx: &mut MockTransaction,
            _username: &str,
        ) -> Result<Vec<FamilyEntity>, sqlx::Error> {
            Ok(self.families.clone())
        }
    }

    fn make_state() -> web::Data<ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>> {
        web::Data::new(ActixState {
            db_connection: MockPoolPostgres,
            oidc_config: dummy_oidc_config(),
            oidc_client: None,
            user_repository: Arc::new(MockUserRepository),
            family_repository: Arc::new(MockFamilyRepository {
                families: vec![
                    FamilyEntity {
                        id: 1,
                        name: "Family A".to_string(),
                    },
                    FamilyEntity {
                        id: 2,
                        name: "Family B".to_string(),
                    },
                ],
            }),
        })
    }

    #[actix_web::test]
    async fn should_return_families_of_username() {
        let state = make_state();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route(
                    "/families",
                    web::get().to(|state: web::Data<ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>>| async move {
                        get_families_from_username(state, Some("JohnDoe".to_string())).await
                    }),
                ),
        )
        .await;

        let req = test::TestRequest::get().uri("/families").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK);
        let body = test::read_body(resp).await;
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains("\"name\":\"Family A\""));
        assert!(body_str.contains("\"name\":\"Family B\""));
    }
}
