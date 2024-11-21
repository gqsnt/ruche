use leptos::server_fn::serde::{Deserialize, Serialize};
use leptos::Params;
use leptos_router::params::Params;

pub mod platform_type_page;
pub mod summoner_page;
pub mod components;


#[derive(Params, PartialEq, Serialize, Deserialize, Clone, Debug, Default)]
pub struct MatchFiltersSearch {
    pub queue_id: Option<i32>,
    pub champion_id: Option<i32>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl MatchFiltersSearch {
    #[cfg(feature = "ssr")]
    pub fn start_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(self.start_date.clone())
    }

    #[cfg(feature = "ssr")]
    pub fn end_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(self.end_date.clone())
    }

    pub fn from_signals(queue_id: Option<String>, champion_id: Option<String>, start_date: Option<String>, end_date: Option<String>) -> Self {
        Self {
            queue_id: queue_id.map(|x| x.parse::<i32>().unwrap_or_default()),
            champion_id: champion_id.map(|x| x.parse::<i32>().unwrap_or_default()),
            start_date,
            end_date,
        }
    }
}
