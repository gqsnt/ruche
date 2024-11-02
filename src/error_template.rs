use std::sync::Arc;
use http::status::StatusCode;
use leptos::*;
use leptos::prelude::ServerFnError;
use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[cfg(feature = "ssr")]
pub type ServerResult<T> = Result<T, ServerFnError>;


#[derive(Clone, Debug, Error)]
pub enum AppError {
    #[error("Not Found")]
    NotFound,
    #[cfg(feature = "ssr")]
    #[error("SQL Error: {0}")]
    SqlxError(Arc<sqlx::Error>),
    #[cfg(feature = "ssr")]
    #[error("Riven Error: {0}")]
    RivenError(Arc<riven::RiotApiError>),
    #[cfg(feature = "ssr")]
    #[error("Custom Error: {0}")]
    CustomError(String),
}

#[cfg(feature = "ssr")]
impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::SqlxError(Arc::new(e))
    }
}

#[cfg(feature = "ssr")]
impl From<riven::RiotApiError> for AppError {
    fn from(e: riven::RiotApiError) -> Self {
        AppError::RivenError(Arc::new(e))
    }
}


impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            #[cfg(feature = "ssr")]
            AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "ssr")]
            AppError::RivenError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            #[cfg(feature = "ssr")]
            AppError::CustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    #[cfg(feature = "ssr")]
    pub fn to_server_fn_error(&self) -> ServerFnError {
        ServerFnError::new(self.to_string())
    }

}
