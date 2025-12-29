use crate::security::oidc::{OidcAdminConfig, OidcClientConfig, OidcConfig};

pub fn dummy_oidc_config() -> OidcConfig {
    OidcConfig {
        url: "http://localhost".to_string(),
        realm_name: "realm".to_string(),
        redirect_uri: "http://localhost/callback".to_string(),
        logout_uri: "http://localhost/logout".to_string(),
        client: OidcClientConfig {
            id: "client_id".to_string(),
            secret: "client_secret".to_string(),
        },
        nonce: None,
        session_timeout_minutes: None,
        admin: OidcAdminConfig {
            client: OidcClientConfig {
                id: "admin_client".to_string(),
                secret: "admin_secret".to_string(),
            },
            create_user_url: "http://localhost/create".to_string(),
        },
    }
}