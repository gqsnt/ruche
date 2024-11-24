use leptos::Params;
use leptos_router::params::{Params};

use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use std::fmt::Debug;
pub mod components;
pub mod platform_type_page;
pub mod summoner_page;

#[derive(Params, PartialEq, Clone, Default)]
pub struct MatchFiltersSearch {
    pub queue_id: Option<u8>,
    pub champion_id: Option<u16>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Archive,Serialize, Deserialize, Default, PartialEq, Clone, Copy)]
pub struct BackEndMatchFiltersSearch {
    pub queue_id: Option<u8>,
    pub champion_id: Option<u16>,
    pub start_date: Option<(u16, u8, u8)>,
    pub end_date: Option<(u16, u8, u8)>,
}

impl BackEndMatchFiltersSearch {
    #[cfg(feature = "ssr")]
    pub fn start_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(
            self.start_date
                .map(|x| format!("{:04}-{:02}-{:02}", x.0, x.1, x.2)),
        )
    }

    #[cfg(feature = "ssr")]
    pub fn end_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(
            self.end_date
                .map(|x| format!("{:04}-{:02}-{:02}", x.0, x.1, x.2)),
        )
    }
    pub fn from_signals(
        queue_id: Option<String>,
        champion_id: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Self {
        Self {
            queue_id: queue_id.map(|x| x.parse::<u8>().unwrap_or_default()),
            champion_id: champion_id.map(|x| x.parse::<u16>().unwrap_or_default()),
            start_date: parse_date(start_date),
            end_date: parse_date(end_date),
        }
    }
}
pub fn parse_date(date: Option<String>) -> Option<(u16, u8, u8)> {
    date.and_then(|date| {
        let date = date.split('-').collect::<Vec<_>>();
        if date.len() == 3 {
            Some((
                date[0].parse().unwrap_or_default(),
                date[1].parse().unwrap_or_default(),
                date[2].parse().unwrap_or_default(),
            ))
        } else {
            None
        }
    })
}
