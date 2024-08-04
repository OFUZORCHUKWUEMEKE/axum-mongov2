use axum::{
    http::StatusCode,
    response::{IntoResponse,Response}
};
use thiserror::Error;

#[derive(Error,Debug)]
pub enum AppError{
    #[error("MongoDb error: {0}")]
    MongoDb(#[from] mongodb::error::Error),
    #[error("Authentication error:{0}")]
    Auth(String),
    #[error("Not found :{0}")]
    NotFound(String),
    #[error("Bad Request :{0}")]
    BadRequest(String),
}

impl IntoResponse for AppError{
    fn into_response(self) -> Response {
        let (status,error_message) = match self{
            AppError::Auth(ref e)=>(StatusCode::UNAUTHORIZED,e.to_string()),
            AppError::BadRequest(ref e)=> (StatusCode::BAD_REQUEST,e.to_string()),
            AppError::MongoDb(ref e)=>(StatusCode::INTERNAL_SERVER_ERROR,e.to_string()),
            AppError::NotFound(ref e )=>(StatusCode::NOT_FOUND,e.to_string())
        };
        (status,error_message).into_response()
    }
}