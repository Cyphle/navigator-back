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
    let mut cors = Cors::default()
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
        .max_age(Some(cors_config.max_age as usize));

    if cors_config.supports_credentials {
        cors = cors.supports_credentials();
    }

    if let Some(additional_headers) = &cors_config.additional_headers {
        for header in additional_headers.split(',') {
            let header = header.trim();
            if !header.is_empty() {
                cors = cors.allowed_header(header);
            }
        }
    }

    cors
}
