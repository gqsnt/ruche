use leptos::{component, view, IntoView, Params};
use leptos_router::params::Params;
use leptos::prelude::{Children, ClassAttribute};
use leptos::prelude::AriaAttributes;

use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use std::fmt::{Debug, Formatter};
use leptos::either::Either;
use leptos::prelude::Show;

pub mod components;
pub mod platform_type_page;
pub mod summoner_page;


#[component]
pub fn ImgSrc(
    #[prop(optional)]
    src:Option<String>,
    #[prop(optional)]
    alt:Option<String>,
    height:u16,
    width:u16,
    #[prop(optional)]
    class:Option<String>,
    #[prop(optional)]
    children:Option<Children>
) -> impl IntoView {
    let class_ = class.unwrap_or_default();
    let src_ = src.unwrap_or_default();
    let alt_ = alt.unwrap_or_default();
    view! {
        <img height=height width=width class=class_ src=src_ alt=alt_ />
        {match children {
            Some(c) => Either::Left(c()),
            None => Either::Right(()),
        }}
    }
}

#[component]
pub fn ImgOptSrc<W:Fn() -> bool + Send + Sync + 'static>(
    when : W,
    #[prop(optional)]
    src:Option<String>,
    #[prop(optional)]
    alt:Option<String>,
    height:u16,
    width:u16,
    #[prop(optional)]
    class:Option<String>,
      #[prop(optional)]
    children:Option<Children>
) -> impl IntoView {
    let class_ = class.unwrap_or_default();
    let src_ = src.unwrap_or_default();
    let alt_ = alt.unwrap_or_default();
    view! {
        <Show when>
            <img height=height width=width class=class_.clone() src=src_.clone() alt=alt_.clone() />

        </Show>
        {match children {
            Some(c) => Either::Left(c()),
            None => Either::Right(()),
        }}
    }
}




#[component]
pub fn ImgBg(
    #[prop(optional)]
    alt:Option<String>,
    #[prop(optional)]
    class:Option<String>,
    #[prop(optional)]
    children:Option<Children>
) -> impl IntoView {
    let class_ = class.unwrap_or_default();
    let alt_ = alt.unwrap_or_default();
    view! {
        <div class=class_ aria-label=alt_ />
        {match children {
            Some(c) => Either::Left(c()),
            None => Either::Right(()),
        }}
    }
}


#[component]
pub fn ImgOptBg< W:Fn() -> bool + Send + Sync + 'static>(
    when : W,
    #[prop(optional)]
    alt:Option<String>,
    #[prop(optional)]
    class:Option<String>,
    #[prop(optional)]
    children:Option<Children>
) -> impl IntoView {
    let class_ = class.unwrap_or_default();
    let alt_ = alt.unwrap_or_default();
    view! {
        <Show when>
            <div class=class_.clone() aria-label=alt_.clone() />

        </Show>
        {match children {
            Some(c) => Either::Left(c()),
            None => Either::Right(()),
        }}
    }
}





#[derive(Params, PartialEq, Clone, Default)]
pub struct MatchFiltersSearch {
    pub queue_id: Option<u8>,
    pub champion_id: Option<u16>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

#[derive(Debug, Archive, Serialize, Deserialize, Default, PartialEq, Clone, Copy)]
pub struct BackEndMatchFiltersSearch {
    pub start_date: Option<CompactDate>,
    pub end_date: Option<CompactDate>,
    pub champion_id: Option<u16>,
    pub queue_id: Option<u8>,
}

impl BackEndMatchFiltersSearch {
    #[cfg(feature = "ssr")]
    pub fn start_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(self.start_date.map(|x| x.to_string()))
    }

    #[cfg(feature = "ssr")]
    pub fn end_date_to_naive(&self) -> Option<chrono::NaiveDateTime> {
        crate::backend::ssr::parse_date(self.end_date.map(|x| x.to_string()))
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
pub fn parse_date(date: Option<String>) -> Option<CompactDate> {
    date.and_then(|date| {
        let date = date.split('-').collect::<Vec<_>>();
        if date.len() == 3 {
            CompactDate::new(
                date[0].parse().unwrap_or_default(),
                date[1].parse().unwrap_or_default(),
                date[2].parse().unwrap_or_default(),
            )
        } else {
            None
        }
    })
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Archive, Serialize, Deserialize)]
pub struct CompactDate(u16);

impl CompactDate {
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        // Year: 7 bits (2000-2127)
        // Month: 4 bits (1-12)
        // Day: 5 bits (1-31)
        if (2000..=2127).contains(&year) && (1..=12).contains(&month) && (1..=31).contains(&day) {
            let y = year - 2000;
            let m = month - 1;
            let d = day - 1;
            let value = (y << 9) | ((m as u16) << 5) | (d as u16);
            Some(CompactDate(value))
        } else {
            None
        }
    }
}

impl std::fmt::Display for CompactDate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let y = ((self.0 >> 9) & 0x7F) + 2000;
        let m = ((self.0 >> 5) & 0x0F) + 1;
        let d = (self.0 & 0x1F) + 1;
        write!(f, "{:04}-{:02}-{:02}", y, m, d)
    }
}
