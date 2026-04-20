use crate::config::actix::ActixState;
use crate::domains::magic_list::http::magic_list_item_middleware::{add_item_to_magic_list_middleware, update_item_of_magic_list_middleware};
use crate::domains::magic_list::http::magic_list_middleware::{create_magic_list_middleware, get_magic_list_summary_middleware};
use crate::domains::magic_list::http::magic_list_requests::{CreateMagicListItemRequest, CreateMagicListRequest, UpdateMagicListItemRequest};
use crate::domains::magic_list::usecases::add_item_to_magic_list_use_case::add_item_to_magic_list_use_case;
use crate::domains::magic_list::usecases::create_magic_list_use_case::create_magic_list_use_case;
use crate::domains::magic_list::usecases::get_magic_list_summary_use_case::get_magic_list_summary_use_case;
use crate::domains::magic_list::usecases::update_item_of_magic_list_use_case::update_item_of_magic_list_use_case;
use actix_session::Session;
use actix_web::{get, post, put, web, Responder};
use log::debug;

#[get("/families/{family_id}/magic-lists/summary")]
pub async fn get_magic_list_summary_endpoint(
    session: Session,
    state: web::Data<ActixState>,
    family_id: web::Path<i32>,
) -> impl Responder {
    debug!("[Controller] Get magic_list summary of family {}", family_id);
    get_magic_list_summary_middleware(
        session,
        state,
        family_id.into_inner(),
        get_magic_list_summary_use_case,
    ).await
}

#[post("/families/{family_id}/magic-lists")]
pub async fn create_magic_list_endpoint(
    payload: web::Json<CreateMagicListRequest>,
    session: Session,
    state: web::Data<ActixState>,
    family_id: web::Path<i32>
) -> impl Responder {
    debug!("[Controller] Create magic list for family {}", family_id);
    create_magic_list_middleware(
        session,
        state,
        family_id.into_inner(),
        payload.into_inner(),
        create_magic_list_use_case
    ).await
}

#[post("/families/{family_id}/magic-lists/{magic_list_id}/items")]
pub async fn add_item_to_magic_list_endpoint(
    payload: web::Json<CreateMagicListItemRequest>,
    session: Session,
    state: web::Data<ActixState>,
    path: web::Path<(i32, i32)>,
) -> impl Responder {
    let (family_id, magic_list_id) = path.into_inner();
    debug!("[Controller] Create item in magic list {} for family {}", magic_list_id, family_id);
    add_item_to_magic_list_middleware(
        session,
        state,
        magic_list_id,
        payload.into_inner(),
        add_item_to_magic_list_use_case
    ).await
}

#[put("/families/{family_id}/magic-lists/{magic_list_id}/items/{item_id}")]
pub async fn update_item_of_magic_list_endpoint(
    payload: web::Json<UpdateMagicListItemRequest>,
    session: Session,
    state: web::Data<ActixState>,
    path: web::Path<(i32, i32, i32)>,
) -> impl Responder {
    let (family_id, magic_list_id, item_id) = path.into_inner();
    debug!("[Controller] Update item {} in magic list {} for family {}", item_id, magic_list_id, family_id);
    update_item_of_magic_list_middleware(
        session,
        state,
        magic_list_id,
        item_id,
        payload.into_inner(),
        update_item_of_magic_list_use_case
    ).await
}