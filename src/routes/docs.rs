use utoipa::{
    OpenApi,
    openapi::{self, Contact},
};

use crate::models::user::{LoginUser, RegisterUser};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::controllers::user::api_register_user,
        crate::controllers::user::api_login_user,
    ),
    components(schemas(RegisterUser, LoginUser))
)]
pub struct ApiDoc;

pub fn get_api_docs() -> openapi::OpenApi {
    let mut docs = ApiDoc::openapi();
    let mut contact = Contact::new();
    contact.email = Some("hadsonramalho@gmail.com".to_string());
    contact.name = Some("Hadson Ramalho".to_string());
    contact.url = Some("https://github.com/HadsonRamalho".to_string());
    docs.info.contact = Some(contact);
    docs.info.license = None;
    docs.info.title = "Rust Backend Template".to_string();
    docs.info.version = "0.1.0".to_string();
    docs.info.extensions = None;
    docs.info.terms_of_service = None;
    docs.external_docs = None;

    docs
}
