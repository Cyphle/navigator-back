use crate::security::oidc::OidcConfig;
use openid::{Bearer, Client, Discovered, StandardClaims};
use std::sync::Arc;
use std::{collections::HashMap, sync::Mutex};
use sqlx::{Pool, Postgres};

pub struct ActixState {
    pub db_connection: &'static Pool<Postgres>,

    pub oidc_config: OidcConfig,
    pub oidc_client: Option<Arc<Mutex<Client<Discovered, StandardClaims>>>>,
}
