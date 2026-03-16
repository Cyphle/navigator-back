use crate::application::errors::ApplicationErrors;
use crate::config::actix::{ActixState, DbConnection};
use crate::domain::dashboard::dashboard::{empty, Dashboard};
use crate::repositories::family::FamilyRepository;
use crate::repositories::user::UserRepository;
use actix_web::web;

pub async fn get_dashboard<DB, U, F>(
    state: web::Data<ActixState<DB, U, F>>,
    username: String,
    family_id: String
) -> Result<Dashboard, ApplicationErrors>
where
    DB: DbConnection,
    U: for<'a> UserRepository<<DB as DbConnection>::Tx<'a>>,
    F: for<'a> FamilyRepository<<DB as DbConnection>::Tx<'a>>,
{
    Ok(empty())
}

#[cfg(test)]
mod tests {
    use crate::application::dashboard::get_dashboard;
    use crate::domain::dashboard::dashboard::empty;
    use crate::testing::actix::mock_state::{mock_actix_state, MockActixState, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;
    use actix_web::web;

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
        let result = get_dashboard(state, "john.doe".to_string(), "Doe family".to_string()).await;

        let dashboard = result.expect("error fetching dashboard");
        assert_eq!(dashboard, empty());
    }
}