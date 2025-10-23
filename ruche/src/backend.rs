use leptos::prelude::ServerFnError;

pub mod server_fns;

#[cfg(feature = "ssr")]
pub mod tasks;

#[cfg(feature = "ssr")]
pub mod live_game_cache;
#[cfg(feature = "ssr")]
pub mod task_director;

pub type ServerResult<T> = Result<T, ServerFnError>;

#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::utils::DurationSince;
    use chrono::{NaiveDateTime, Utc};
    use common::consts::platform_route::PlatformRoute;
    use http::status::StatusCode;
    use leptos::prelude::ServerFnError;
    use std::fmt::Formatter;
    use std::num::ParseIntError;
    use std::sync::Arc;
    use thiserror::Error;

    pub type AppResult<T> = Result<T, AppError>;

    // impl From<AppError> for ServerFnError {
    //     fn from(e: AppError) -> Self {
    //         e.to_server_fn_error()
    //     }
    // }

    pub fn format_duration_since(date_time: NaiveDateTime) -> DurationSince {
        let now = Utc::now().naive_utc();
        let seconds = (now - date_time).num_seconds();

        DurationSince::new(
            match seconds {
                0..=59 => format!("{} seconds ago", seconds),
                60..=3599 => format!("{} minutes ago", seconds / 60),
                3600..=86_399 => format!("{} hours ago", seconds / 3600),
                86_400..=2_592_000 => format!("{} days ago", seconds / 86_400),
                2_592_001..=31_536_000 => format!("{} months ago", seconds / 2_592_000), // Approx 30 days per month
                _ => format!("{} years ago", seconds / 31_536_000), // Approx 365 days per year
            }
            .as_str(),
        )
    }

    pub fn parse_date(date: Option<String>) -> Option<NaiveDateTime> {
        date.as_deref().and_then(|s| {
            if s.is_empty() {
                None
            } else {
                NaiveDateTime::parse_from_str(&format!("{} 00:00:00", s), "%Y-%m-%d %H:%M:%S").ok()
            }
        })
    }

    #[derive(sqlx::FromRow)]
    pub struct Id {
        pub id: i32,
    }

    #[derive(Clone, Debug, Error)]
    pub enum AppError {
        #[error("Not Found")]
        NotFound,
        #[error("SQL Error: {0}")]
        SqlxError(Arc<sqlx::Error>),
        #[error("Sitemap Error: {0}")]
        SiteMapError(Arc<sitemap::Error>),
        #[error("Riven Error: {0}")]
        RivenError(Arc<riven::RiotApiError>),
        #[error("Reqwest Error: {0}")]
        ReqwestError(Arc<reqwest::Error>),
        #[error("Parse Error: {0}")]
        ParseIntError(ParseIntError),
        #[error("Serde json Error: {0}")]
        SerdeJsonError(Arc<serde_json::Error>),
        #[error("Custom Error: {0}")]
        CustomError(String),
        #[error("chrono Error: {0}")]
        ChronoError(Arc<chrono::ParseError>),
        #[error("Std Io Error: {0}")]
        StdIoError(Arc<std::io::Error>),
    }

    impl From<std::io::Error> for AppError {
        fn from(e: std::io::Error) -> Self {
            AppError::StdIoError(Arc::new(e))
        }
    }

    impl From<serde_json::Error> for AppError {
        fn from(e: serde_json::Error) -> Self {
            AppError::SerdeJsonError(Arc::new(e))
        }
    }

    impl From<chrono::ParseError> for AppError {
        fn from(e: chrono::ParseError) -> Self {
            AppError::ChronoError(Arc::new(e))
        }
    }

    impl From<ParseIntError> for AppError {
        fn from(e: ParseIntError) -> Self {
            AppError::ParseIntError(e)
        }
    }

    impl From<sqlx::Error> for AppError {
        fn from(e: sqlx::Error) -> Self {
            AppError::SqlxError(Arc::new(e))
        }
    }

    impl From<riven::RiotApiError> for AppError {
        fn from(e: riven::RiotApiError) -> Self {
            AppError::RivenError(Arc::new(e))
        }
    }

    impl From<reqwest::Error> for AppError {
        fn from(e: reqwest::Error) -> Self {
            AppError::ReqwestError(Arc::new(e))
        }
    }

    impl From<sitemap::Error> for AppError {
        fn from(e: sitemap::Error) -> Self {
            AppError::SiteMapError(Arc::new(e))
        }
    }

    impl AppError {
        pub fn status_code(&self) -> StatusCode {
            match self {
                AppError::NotFound => StatusCode::NOT_FOUND,
                AppError::SqlxError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::RivenError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::ReqwestError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::SiteMapError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::CustomError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::ParseIntError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::SerdeJsonError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::ChronoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                AppError::StdIoError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            }
        }

        pub fn to_server_fn_error(&self) -> ServerFnError {
            ServerFnError::new(self.to_string())
        }
        pub fn as_server_fn_error<T>(&self) -> Result<T, ServerFnError> {
            Err(ServerFnError::new(self.to_string()))
        }
    }

    #[derive(
        Clone, Copy, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Serialize, serde::Deserialize,
    )]
    #[sqlx(type_name = "platform_type")]
    pub enum PlatformRouteDb {
        BR,
        EUNE,
        EUW,
        JP,
        KR,
        LAN,
        LAS,
        MENA,
        NA,
        OCE,
        PH,
        RU,
        SG,
        TH,
        TR,
        TW,
        VN,
        PBE,
    }

    impl From<PlatformRouteDb> for PlatformRoute {
        fn from(value: PlatformRouteDb) -> Self {
            match value {
                PlatformRouteDb::BR => PlatformRoute::BR,
                PlatformRouteDb::EUNE => PlatformRoute::EUNE,
                PlatformRouteDb::EUW => PlatformRoute::EUW,
                PlatformRouteDb::JP => PlatformRoute::JP,
                PlatformRouteDb::KR => PlatformRoute::KR,
                PlatformRouteDb::LAN => PlatformRoute::LAN,
                PlatformRouteDb::LAS => PlatformRoute::LAS,
                PlatformRouteDb::MENA => PlatformRoute::MENA,
                PlatformRouteDb::NA => PlatformRoute::NA,
                PlatformRouteDb::OCE => PlatformRoute::OCE,
                PlatformRouteDb::PH => PlatformRoute::PH,
                PlatformRouteDb::RU => PlatformRoute::RU,
                PlatformRouteDb::SG => PlatformRoute::SG,
                PlatformRouteDb::TH => PlatformRoute::TH,
                PlatformRouteDb::TR => PlatformRoute::TR,
                PlatformRouteDb::TW => PlatformRoute::TW,
                PlatformRouteDb::VN => PlatformRoute::VN,
                PlatformRouteDb::PBE => PlatformRoute::PBE,
            }
        }
    }

    impl From<PlatformRoute> for PlatformRouteDb {
        fn from(value: PlatformRoute) -> Self {
            match value {
                PlatformRoute::BR => PlatformRouteDb::BR,
                PlatformRoute::EUNE => PlatformRouteDb::EUNE,
                PlatformRoute::EUW => PlatformRouteDb::EUW,
                PlatformRoute::JP => PlatformRouteDb::JP,
                PlatformRoute::KR => PlatformRouteDb::KR,
                PlatformRoute::LAN => PlatformRouteDb::LAN,
                PlatformRoute::LAS => PlatformRouteDb::LAS,
                PlatformRoute::MENA => PlatformRouteDb::MENA,
                PlatformRoute::NA => PlatformRouteDb::NA,
                PlatformRoute::OCE => PlatformRouteDb::OCE,
                PlatformRoute::PH => PlatformRouteDb::PH,
                PlatformRoute::RU => PlatformRouteDb::RU,
                PlatformRoute::SG => PlatformRouteDb::SG,
                PlatformRoute::TH => PlatformRouteDb::TH,
                PlatformRoute::TR => PlatformRouteDb::TR,
                PlatformRoute::TW => PlatformRouteDb::TW,
                PlatformRoute::VN => PlatformRouteDb::VN,
                PlatformRoute::PBE => PlatformRouteDb::PBE,
            }
        }
    }

    impl PlatformRouteDb {
        pub fn from_raw_str(str: &str) -> Self {
            PlatformRoute::from_code(str).unwrap_or_default().into()
        }
    }

    impl std::fmt::Display for PlatformRouteDb {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", PlatformRoute::from(*self))
        }
    }
}
