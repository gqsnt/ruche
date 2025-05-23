use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_summoner::get_summoner;
use crate::backend::server_fns::update_summoner::UpdateSummoner;
use crate::utils::{summoner_url, ProPlayerSlug, SSEEvent};
use crate::views::summoner_page::summoner_nav::SummonerNav;
use crate::views::{ImgSrc, PendingLoading};
use common::consts::platform_route::PlatformRoute;
use common::consts::profile_icon::ProfileIcon;
use common::consts::HasStaticSrcAsset;
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::Read;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos_router::hooks::use_params_map;
use serde::{Deserialize, Serialize};

pub mod match_details;
pub mod summoner_champions_page;
pub mod summoner_encounter_page;
pub mod summoner_encounters_page;
pub mod summoner_live_page;
pub mod summoner_matches_page;
pub mod summoner_nav;
pub mod summoner_search_page;

#[component]
pub fn SummonerPage() -> impl IntoView {
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();

    let params = use_params_map();

    let platform_type = move || {
        params
            .read()
            .get("platform_type")
            .clone()
            .unwrap_or_default()
    };
    let summoner_slug = move || {
        params
            .read()
            .get("summoner_slug")
            .clone()
            .unwrap_or_default()
    };

    // Update the summoner signal when resource changes
    let summoner_resource = leptos::server::Resource::new_bincode_blocking(
        move || (platform_type(), summoner_slug()),
        |(platform, summoner_slug)| async move {
            get_summoner(PlatformRoute::from(platform.as_str()), summoner_slug).await
        },
    );

    let summoner_view = move || {
        Suspend::new(async move {
            match summoner_resource.await {
                Ok(summoner) => Either::Left({
                    let (level_signal, set_level) = signal(summoner.summoner_level);
                    let (profile_icon_signal, set_profile_icon) = signal(summoner.profile_icon_id);
                    let (sse_match_update_version, set_sse_match_update_version) =
                        signal(None::<SSEMatchUpdateVersion>);
                    let (sse_in_live_game, set_sse_in_live_game) = signal(SSEInLiveGame::default());

                    provide_context(sse_match_update_version);
                    provide_context(sse_in_live_game);
                    provide_context(summoner.clone());

                    let update_summoner_action = ServerAction::<UpdateSummoner>::new();
                    let (pending, set_pending) = signal(false);
                    Effect::new(move |_| {
                        let _ = update_summoner_action.version().get();
                        set_pending(false);
                        if let Some(Ok(Some((level, profile_icon_id)))) =
                            update_summoner_action.value().get()
                        {
                            if level != level_signal() {
                                set_level(level);
                            }
                            if profile_icon_id != profile_icon_signal() {
                                set_profile_icon(profile_icon_id);
                            }
                        }
                    });

                    #[cfg(not(feature = "ssr"))]
                    let sse_event_signal = {
                        use futures::StreamExt;
                        use send_wrapper::SendWrapper;
                        let mut source = SendWrapper::new(
                            gloo_net::eventsource::futures::EventSource::new(
                                format!(
                                    "/sse/match_updated/{}/{}",
                                    summoner.platform.to_string(),
                                    summoner.id
                                )
                                .as_str(),
                            )
                            .expect("couldn't connect to SSE stream"),
                        );
                        let s = ReadSignal::from_stream_unsync(
                            source
                                .subscribe("message")
                                .expect("couldn't subscribe to SSE stream")
                                .filter_map(|value| async move {
                                    value
                                        .map(|(_, message_event)| {
                                            SSEEvent::from_string(
                                                message_event
                                                    .data()
                                                    .as_string()
                                                    .expect("failed to parse sse string")
                                                    .as_str(),
                                            )
                                            .ok()
                                        })
                                        .ok()
                                        .flatten()
                                }),
                        );
                        on_cleanup(move || source.take().close());
                        s
                    };
                    #[cfg(feature = "ssr")]
                    let (sse_event_signal, _) = signal(None::<SSEEvent>);

                    Effect::new(move |_| {
                        let event = sse_event_signal.get();
                        match event {
                            Some(SSEEvent::SummonerMatches(version)) => {
                                set_sse_match_update_version(Some(SSEMatchUpdateVersion(version)));
                            }
                            Some(SSEEvent::LiveGame(version)) => {
                                set_sse_in_live_game(SSEInLiveGame(version));
                            }
                            _ => {}
                        }
                    });

                    meta_store
                        .image()
                        .set(ProfileIcon(summoner.profile_icon_id).get_static_asset_url());
                    view! {
                        <div class="flex justify-center">
                            <div class="flex w-[768px] my-2 space-x-2">
                                <SummonerInfo
                                    game_name=summoner.game_name.clone()
                                    tag_line=summoner.tag_line.clone()
                                    pro_slug=summoner.pro_slug
                                    platform=summoner.platform
                                    level_signal=level_signal
                                    profile_icon_signal=profile_icon_signal
                                />
                                <div class="h-fit">

                                    <button
                                        class="my-button flex items-center"
                                        on:click=move |e| {
                                            e.prevent_default();
                                            set_pending(true);
                                            update_summoner_action
                                                .dispatch(UpdateSummoner {
                                                    summoner_id: summoner.id,
                                                    game_name: summoner.game_name.clone(),
                                                    tag_line: summoner.tag_line.clone(),
                                                    platform_route: summoner.platform,
                                                });
                                        }
                                    >
                                        <PendingLoading pending>Update</PendingLoading>
                                    </button>

                                </div>
                            </div>
                        </div>

                        <SummonerNav />
                    }
                }),
                Err(_) => Either::Right(()),
            }
        })
    };

    view! {
        <Transition fallback=move || {
            view! { <div class="text-center">Loading Summoner</div> }
        }>{summoner_view}</Transition>
    }
}

#[component]
pub fn SummonerInfo(
    game_name: String,
    tag_line: String,
    platform: PlatformRoute,
    pro_slug: Option<ProPlayerSlug>,
    #[prop(into)] level_signal: ReadSignal<u16>,
    #[prop(into)] profile_icon_signal: ReadSignal<u16>,
    #[prop(default = true)] is_self: bool,
) -> impl IntoView {
    view! {
        <div class="flex item-center max-w-[280px]" class=("flex-row-reverse", !is_self)>
            {move || {
                view! {
                    <ImgSrc
                        alt=ProfileIcon(profile_icon_signal()).to_string()
                        src=ProfileIcon(profile_icon_signal()).get_static_asset_url()
                        width=64
                        height=64
                        class="w-16 h-16".to_string()
                    />
                }
            }}
            <div
                class="flex flex-col items-start "
                class=("ml-2", is_self)
                class=("mr-2", !is_self)
            >
                <a href=summoner_url(
                    platform.as_ref(),
                    game_name.as_ref(),
                    tag_line.as_ref(),
                )>{game_name.clone()}#{tag_line.clone()}</a>
                <div class="flex ">
                    <span>lvl. {move || level_signal()}</span>
                    {pro_slug
                        .map(|pps| {
                            view! {
                                <a
                                    target="_blank"
                                    href=format!("https://lolpros.gg/player/{}", pps.as_ref())
                                    class=" bg-purple-800 rounded px-1 py-0.5 text-center ml-1"
                                >
                                    PRO
                                </a>
                            }
                        })}
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Summoner {
    pub id: i32,
    pub game_name: String,
    pub pro_slug: Option<ProPlayerSlug>,
    pub tag_line: String,
    pub profile_icon_id: u16,
    pub summoner_level: u16,
    pub platform: PlatformRoute,
}

impl Summoner {
    pub fn to_route_path(&self) -> String {
        summoner_url(
            self.platform.as_ref(),
            self.game_name.as_ref(),
            self.tag_line.as_ref(),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub struct SSEMatchUpdateVersion(pub u16);

#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub struct SSEInLiveGame(pub Option<u16>);
