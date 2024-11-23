use leptos::prelude::ServerFnError;

pub mod server_fns;

#[cfg(feature = "ssr")]
pub mod updates;

#[cfg(feature = "ssr")]
pub mod lol_static;

#[cfg(feature = "ssr")]
pub mod generate_sitemap;
#[cfg(feature = "ssr")]
pub mod live_game_cache;


pub type ServerResult<T> = Result<T, ServerFnError>;

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::fmt::Formatter;
    use crate::consts::platform_route::PlatformRoute;
    use chrono::{NaiveDateTime, Utc};
    use http::status::StatusCode;
    use leptos::prelude::ServerFnError;
    use std::num::ParseIntError;
    use std::sync::Arc;
    use thiserror::Error;

    pub type AppResult<T> = Result<T, AppError>;

    // impl From<AppError> for ServerFnError {
    //     fn from(e: AppError) -> Self {
    //         e.to_server_fn_error()
    //     }
    // }


    pub fn format_duration_since(dt: NaiveDateTime) -> String {
        let match_end = dt.and_utc();
        let now = Utc::now();
        let duration = now.signed_duration_since(match_end);

        if duration.num_days() > 0 {
            format!("{} days ago", duration.num_days())
        } else if duration.num_hours() > 0 {
            format!("{} hours ago", duration.num_hours())
        } else if duration.num_minutes() > 0 {
            format!("{} minutes ago", duration.num_minutes())
        } else {
            format!("{} seconds ago", duration.num_seconds())
        }
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
        #[error("Ravif: {0}")]
        RavifError(Arc<ravif::Error>),
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

    impl From<ravif::Error> for AppError {
        fn from(e: ravif::Error) -> Self {
            AppError::RavifError(Arc::new(e))
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
                AppError::RavifError(_) => StatusCode::INTERNAL_SERVER_ERROR,
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

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd, sqlx::Type, serde::Serialize, serde::Deserialize)]
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
                PlatformRouteDb::BR => PlatformRoute::BR1,
                PlatformRouteDb::EUNE => PlatformRoute::EUN1,
                PlatformRouteDb::EUW => PlatformRoute::EUW1,
                PlatformRouteDb::JP => PlatformRoute::JP1,
                PlatformRouteDb::KR => PlatformRoute::KR,
                PlatformRouteDb::LAN => PlatformRoute::LA1,
                PlatformRouteDb::LAS => PlatformRoute::LA2,
                PlatformRouteDb::MENA => PlatformRoute::ME1,
                PlatformRouteDb::NA => PlatformRoute::NA1,
                PlatformRouteDb::OCE => PlatformRoute::OC1,
                PlatformRouteDb::PH => PlatformRoute::PH2,
                PlatformRouteDb::RU => PlatformRoute::RU,
                PlatformRouteDb::SG => PlatformRoute::SG2,
                PlatformRouteDb::TH => PlatformRoute::TH2,
                PlatformRouteDb::TR => PlatformRoute::TR1,
                PlatformRouteDb::TW => PlatformRoute::TW2,
                PlatformRouteDb::VN => PlatformRoute::VN2,
                PlatformRouteDb::PBE => PlatformRoute::PBE1,
            }
        }
    }

    impl From<PlatformRoute> for PlatformRouteDb {
        fn from(value: PlatformRoute) -> Self {
            match value {
                PlatformRoute::BR1 => PlatformRouteDb::BR,
                PlatformRoute::EUN1 => PlatformRouteDb::EUNE,
                PlatformRoute::EUW1 => PlatformRouteDb::EUW,
                PlatformRoute::JP1 => PlatformRouteDb::JP,
                PlatformRoute::KR => PlatformRouteDb::KR,
                PlatformRoute::LA1 => PlatformRouteDb::LAN,
                PlatformRoute::LA2 => PlatformRouteDb::LAS,
                PlatformRoute::ME1 => PlatformRouteDb::MENA,
                PlatformRoute::NA1 => PlatformRouteDb::NA,
                PlatformRoute::OC1 => PlatformRouteDb::OCE,
                PlatformRoute::PH2 => PlatformRouteDb::PH,
                PlatformRoute::RU => PlatformRouteDb::RU,
                PlatformRoute::SG2 => PlatformRouteDb::SG,
                PlatformRoute::TH2 => PlatformRouteDb::TH,
                PlatformRoute::TR1 => PlatformRouteDb::TR,
                PlatformRoute::TW2 => PlatformRouteDb::TW,
                PlatformRoute::VN2 => PlatformRouteDb::VN,
                PlatformRoute::PBE1 => PlatformRouteDb::PBE,
            }
        }
    }

    impl PlatformRouteDb {

        pub fn from_raw_str(str: &str) -> Self {
            PlatformRoute::from_raw_str(str).into()
        }
    }

    impl std::fmt::Display for PlatformRouteDb{
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", PlatformRoute::from(*self))
        }
    }
}



