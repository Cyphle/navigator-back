use crate::application::family::{get_families_from_username, FamilyServiceError};
use crate::config::actix::{ActixState, DbConnection};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use crate::security::token::get_username_from_session;
use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use log::{debug, error};
use serde::Serialize;

#[derive(Serialize)]
struct FamilyView {
    name: String,
}

#[get("/families")]
pub async fn get_families(session: Session, state: web::Data<ActixState>) -> impl Responder {
    get_families_with_state(session, state).await
}

async fn get_families_with_state<DB, U, F>(
    session: Session,
    state: web::Data<ActixState<DB, U, F>>,
) -> impl Responder
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    debug!("Getting families");

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
        Err(FamilyServiceError::MissingUsername) => HttpResponse::Unauthorized().finish(),
        Err(FamilyServiceError::Database(e)) => {
            error!("Error getting families: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::actix::{ActixState, DbConnection};
    use crate::domain::user::user::User;
    use crate::repositories::family::{FamilyEntity, FamilyRepository};
    use crate::repositories::user::UserRepository;
    use crate::testing::security::oidc::dummy_oidc_config;
    use actix_web::http::StatusCode;
    use actix_web::{test, web, App, HttpResponse};
    use async_trait::async_trait;
    use spy::{spy, Spy};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;

    // TODO à mettre dans testing
    struct MockPoolPostgres;

    // TODO à mettre dans testing
    struct MockTransaction;

    // TODO à mettre dans testing
    impl DbConnection for MockPoolPostgres {
        type Tx<'a> = MockTransaction;

        fn begin<'a>(
            &'a self,
        ) -> Pin<Box<dyn Future<Output = Result<Self::Tx<'a>, sqlx::Error>> + Send + 'a>> {
            Box::pin(async { Ok(MockTransaction) })
        }
    }

    // TODO à mettre dans testing
    struct MockUserRepository;

    // TODO à mettre dans testing
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

    // TODO à mettre dans testing
    struct MockFamilyRepository {
        families: Vec<FamilyEntity>,
    }

    // TODO à mettre dans testing
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

    // TODO à mettre dans testing et une fonction qui peut recevoir des mock state ou alors un truc compasable par repository
    fn make_state(
    ) -> web::Data<ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>> {
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
    async fn should_return_unauthorized_without_session() {
        let state = make_state();
        let (spy_handler, spy) = spy!();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route(
                    "/families",
                    web::get().to(
                        move |session: actix_session::Session,
                              state: web::Data<
                            ActixState<MockPoolPostgres, MockUserRepository, MockFamilyRepository>,
                        >| {
                            spy_handler();
                            super::get_families_with_state::<
                                MockPoolPostgres,
                                MockUserRepository,
                                MockFamilyRepository,
                            >(session, state)
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
