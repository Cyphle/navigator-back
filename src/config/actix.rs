use crate::security::oidc::OidcConfig;
use openid::{Bearer, Client, Discovered, StandardClaims};
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use sqlx::{Pool, Postgres};
use crate::repositories::user::UserRepository;

pub struct ActixState {
    pub oidc_config: OidcConfig,
    pub oidc_client: Option<Arc<Mutex<Client<Discovered, StandardClaims>>>>,

    pub db_connection: Pool<Postgres>,
    pub user_repository: Arc<dyn UserRepository>,
}
