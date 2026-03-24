use crate::config::actix::{ActixState, DbConnection};
use crate::domains::dashboard::domain::dashboard::{empty, Dashboard};
use crate::domains::family::repositories::family_repository::FamilyRepository;
use crate::domains::user::repositories::user_repository::UserRepository;
use actix_web::web;
use crate::domains::common::errors::errors::ApplicationError;

pub async fn get_dashboard_use_case<DB, U, F>(
    _state: web::Data<ActixState<DB, U, F>>,
    _username: String,
    _family_id: String
) -> Result<Dashboard, Box<dyn ApplicationError>>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    Ok(empty())
}

#[cfg(test)]
mod tests {
    use crate::domains::dashboard::usecases::get_dashboard_use_case::get_dashboard_use_case;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::web;
    use crate::domains::dashboard::domain::dashboard::empty;

    fn state_ok() -> web::Data<MockActixState> {
        mock_actix_state(
            MockPoolPostgres,
            MockStateConfig {
                ..MockStateConfig::default()
            },
        )
    }

    #[actix_web::test]
    async fn should_return_dashboard() {
        let state = state_ok();
        let result = get_dashboard_use_case(state, "john.doe".to_string(), "Doe family".to_string()).await;

        let dashboard = result.expect("error fetching dashboard");
        assert_eq!(dashboard, empty());
    }
}