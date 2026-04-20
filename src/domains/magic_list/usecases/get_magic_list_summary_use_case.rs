use crate::config::actix::{ActixState, DbConnection};
use crate::domains::common::errors::errors::ApplicationError;
use crate::domains::magic_list::domain::magic_list_summary::MagicListSummary;
use actix_web::web;

pub async fn get_magic_list_summary_use_case<DB: DbConnection>(
    state: web::Data<ActixState<DB>>,
    username: String,
    family_id: i32,
) -> Result<Vec<MagicListSummary>, Box<dyn ApplicationError>> {
    state.magic_list_repository.get_summary_for_user_and_family(&username, family_id).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domains::common::visibility::Visibility;
    use crate::domains::magic_list::domain::magic_list_type::MagicListType;
    use crate::testing::actix::mock_state::{mock_actix_state, MockMagicListConfig, MockStateConfig};
    use crate::testing::repositories::mock_database::MockPoolPostgres;

    fn a_summary(id: i32, name: &str, visibility: Visibility, magic_list_type: MagicListType, family_id: Option<i32>, item_count: i64) -> MagicListSummary {
        MagicListSummary { id, name: name.to_string(), visibility, magic_list_type, family_id, item_count }
    }

    #[actix_web::test]
    async fn should_return_summaries_for_user_and_family() {
        let summaries = vec![
            a_summary(1, "Courses", Visibility::Shared, MagicListType::Simple, Some(1), 3),
            a_summary(2, "Mon perso", Visibility::Personal, MagicListType::Task, Some(1), 1),
        ];
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig {
            magic_list: MockMagicListConfig {
                summaries: summaries.clone(),
                ..Default::default()
            },
            ..Default::default()
        });

        let result = get_magic_list_summary_use_case(state, "alice".to_string(), 1).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].name, "Courses");
        assert_eq!(result[0].item_count, 3);
        assert_eq!(result[1].name, "Mon perso");
        assert_eq!(result[1].visibility, Visibility::Personal);
    }

    #[actix_web::test]
    async fn should_return_empty_when_no_lists() {
        let state = mock_actix_state(MockPoolPostgres, MockStateConfig::default());

        let result = get_magic_list_summary_use_case(state, "alice".to_string(), 1).await.unwrap();

        assert!(result.is_empty());
    }
}
