use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::common::errors::missing_username_error::MissingUsernameError;
use crate::domains::common::errors::repository_error::RepositoryError;
use crate::domains::family::domain::family_repository::FamilyRepository;
use crate::domains::user::domain::user::User;
use crate::domains::user::domain::user_repository::UserRepository;
use actix_web::web;
use log::debug;

pub async fn get_user_info_use_case(
    state: web::Data<ActixState>,
    username: Option<String>,
) -> Result<User, Box<dyn ApplicationError>>
{
    let username =
        username.ok_or_else(|| Box::new(MissingUsernameError) as Box<dyn ApplicationError>)?;
    debug!("Username in session: {:?}", username);

    let mut tx = state.db_connection.begin().await.map_err(|e| {
        Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>
    })?;

    state
        .user_repository
        .get_user(&mut *tx, &username)  // &mut *tx dereference Transaction en PgConnection
        .await
        .map_err(|e| {
            Box::new(RepositoryError { error: e.to_string() }) as Box<dyn ApplicationError>
        })
}

// TODO revoir les tests et les mocks associés
// #[cfg(test)]
// mod tests {
//     use actix_web::web;
//     use crate::config::actix::ActixState;
//     use crate::security::oidc::OidcConfig;
//
//     fn make_state(user_should_error: bool) -> web::Data<ActixState> {
//         web::Data::new(ActixState {
//             oidc_config: OidcConfig::default(),
//             oidc_client: None,
//             db: PgPoolOptions::new()
//                 .connect_lazy("postgres://dummy/dummy")
//                 .unwrap(),
//             user_repo: Arc::new(MockUserRepository {
//                 should_error: user_should_error,
//             }),
//             family_repo: Arc::new(MockFamilyRepository {
//                 families: vec![],
//                 should_error: false,
//             }),
//         })
//     }
//
//     #[actix_web::test]
//     async fn should_error_when_username_missing() {
//         let result = get_user_info_use_case(make_state(false), None).await;
//         assert!(result.is_err());
//         assert_eq!(result.unwrap_err().get_message(), "No username specified");
//     }
//
//     #[actix_web::test]
//     async fn should_error_on_repository_failure() {
//         let result = get_user_info_use_case(make_state(true), Some("bob".to_string())).await;
//         assert!(
//             result
//                 .unwrap_err()
//                 .get_message()
//                 .contains("no rows returned")
//         );
//     }
//
//     #[actix_web::test]
//     async fn should_return_user() {
//         let result = get_user_info_use_case(make_state(false), Some("carol".to_string())).await;
//         assert_eq!(result.unwrap().username, "mock_user");
//     }
// }
