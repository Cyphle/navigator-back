use actix_session::Session;
use actix_web::{get, web, HttpResponse, Responder};
use actix_web::web::Data;
use log::{error, info};
use openid::{Bearer, Client, Discovered, StandardClaims};
use url::Url;
use crate::config::actix::ActixState;
use crate::config::application::USER_SESSION_KEY;
use crate::security::controllers::auth_request::AuthRequest;

#[get("/logout")]
async fn logout(
    session: Session,
    state: Data<ActixState>,
    _: web::Query<AuthRequest>,
) -> impl Responder {
    log::debug!("Logging out user");

    let user_session = session.get::<Bearer>(USER_SESSION_KEY);
    let client = state.oidc_client.as_ref().unwrap().lock().unwrap();
    let logout_uri: &str = state.oidc_config.logout_uri.as_ref();

    let bearer = match user_session {
        Err(e) => {
            error!("No session repositories found: {}", e);
            return HttpResponse::PermanentRedirect()
                .append_header(("Location", "/"))
                .finish();
        }
        Ok(None) => {
            error!("No session repositories found");
            return HttpResponse::Ok().body("Logged out");
        }
        Ok(Some(b)) => b,
    };

    match build_logout_url(&client, &bearer.clone().id_token.unwrap(), logout_uri).await {
        Ok(logout_url) => {
            session.remove(USER_SESSION_KEY);
            info!("Redirecting to logout URL: {}", logout_url);
            // TODO y a un petit truc qui est pas bien géré pour finir le logout. Mais sinon ça marche
            HttpResponse::PermanentRedirect()
                .append_header(("Location", logout_url.to_string()))
                .finish()
        }
        Err(e) => {
            error!("Error generating logout URL: {}", e);
            session.remove(USER_SESSION_KEY);
            HttpResponse::Ok().body("Logged out")
        }
    }
}

pub async fn build_logout_url(
    client: &Client<Discovered, StandardClaims>,
    id_token: &str,
    logout_uri: &str,
) -> Result<Url, Box<dyn std::error::Error>> {
    // Access the discovered metadata
    let discovered_metadata = client.config();

    // Extract the end_session_endpoint
    let mut end_session_endpoint = discovered_metadata
        .end_session_endpoint
        .as_ref()
        .ok_or("End session endpoint not available in metadata")?
        .clone();

    end_session_endpoint
        .query_pairs_mut()
        .append_pair("id_token_hint", id_token)
        .append_pair("post_logout_redirect_uri", logout_uri);

    Ok(end_session_endpoint.clone())
}
