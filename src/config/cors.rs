use actix_cors::Cors;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origin: String,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub supports_credentials: bool,
    pub max_age: u64,
    pub additional_headers: Option<String>,
}

pub fn actix_cors_config(cors_config: &CorsConfig) -> Cors {
    Cors::default()
        .allowed_origin(cors_config.allowed_origin.as_str())
        .allowed_methods(
            cors_config
                .allowed_methods
                .iter()
                .map(|m| m.parse::<actix_web::http::Method>().unwrap())
                .collect::<Vec<_>>(),
        )
        .allowed_headers(
            cors_config
                .allowed_headers
                .iter()
                .map(|h| {
                    h.parse::<actix_web::http::header::HeaderName>()
                        .unwrap()
                })
                .collect::<Vec<_>>(),
        )
        .supports_credentials() // Optional, if credentials are used
        .max_age(3600)
}