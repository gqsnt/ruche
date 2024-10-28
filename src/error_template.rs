use std::sync::Arc;
use http::status::StatusCode;
use leptos::*;
use thiserror::Error;
#[cfg(feature = "ssr")]
use leptos::ServerFnError;

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
        }
    }

    #[cfg(feature = "ssr")]
    pub fn to_server_fn_error(&self) -> ServerFnError {
        ServerFnError::new(self.to_string())
    }

}

// A basic function to display errors served by the error boundaries.
// Feel free to do more complicated things here than just displaying the error.
#[component]
pub fn ErrorTemplate(
    #[prop(optional)] outside_errors: Option<Errors>,
    #[prop(optional)] errors: Option<RwSignal<Errors>>,
) -> impl IntoView {
    let errors = match outside_errors {
        Some(e) => create_rw_signal(e),
        None => match errors {
            Some(e) => e,
            None => panic!("No Errors found and we expected errors!"),
        },
    };
    // Get Errors from Signal
    let errors = errors.get_untracked();

    // Downcast lets us take a type that implements `std::error::Error`
    let errors: Vec<AppError> = errors
        .into_iter()
        .filter_map(|(_k, v)| v.downcast_ref::<AppError>().cloned())
        .collect();
    println!("Errors: {errors:#?}");

    // Only the response code for the first error is actually sent from the server
    // this may be customized by the specific application
    #[cfg(feature = "ssr")]
    {
        use leptos_axum::ResponseOptions;
        let response = use_context::<ResponseOptions>();
        if let Some(response) = response {
            response.set_status(errors[0].status_code());
        }
    }

    view! {
        <h1>{if errors.len() > 1 {"Errors"} else {"Error"}}</h1>
        <For
            // a function that returns the items we're iterating over; a signal is fine
            each= move || {errors.clone().into_iter().enumerate()}
            // a unique key for each item as a reference
            key=|(index, _error)| *index
            // renders each item to a view
            children=move |error| {
                let error_string = error.1.to_string();
                let error_code= error.1.status_code();
                view! {
                    <h2>{error_code.to_string()}</h2>
                    <p>"Error: " {error_string}</p>
                }
            }
        />
    }
}
