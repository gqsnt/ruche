use crate::app::{to_summoner_identifier_memo, MetaStore, MetaStoreStoreFields, SummonerIdentifier, SummonerRouteParams};
use crate::backend::server_fns::get_summoner::get_summoner;
use crate::backend::server_fns::update_summoner::UpdateSummoner;
use crate::utils::{summoner_url, ProPlayerSlug};
use crate::views::components::match_filters::MatchFilters;
use crate::views::summoner_page::summoner_nav::SummonerNav;
use crate::views::{ImgSrc, PendingLoading, ProPlayerSlugView};
use bitcode::{Decode, Encode};
use common::consts::platform_route::PlatformRoute;
use common::consts::profile_icon::ProfileIcon;
use common::consts::HasStaticSrcAsset;
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::Read;
use leptos::prelude::*;
use leptos::server::codee::binary::BitcodeCodec;
use leptos::{component, view, IntoView};

use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::{lazy_route, LazyRoute};
use crate::views::summoner_page::summoner_search_page::SummonerSearch;

pub mod match_details;
pub mod summoner_champions_page;
pub mod summoner_encounter_page;
pub mod summoner_encounters_page;
pub mod summoner_live_page;
pub mod summoner_matches_page;
pub mod summoner_nav;
pub mod summoner_search_page;

pub struct SummonerPageRoute {
    pub summoner_resource: Resource<Result<Summoner, ServerFnError>, BitcodeCodec>,
    pub summoner_identifier_memo: Memo<SummonerIdentifier>
}

#[lazy_route]
impl LazyRoute for SummonerPageRoute {
    fn data() -> Self {
        let summoner_route_params = use_params::<SummonerRouteParams>();
        let summoner_identifier_memo = to_summoner_identifier_memo(
            summoner_route_params
        );

        let summoner_resource = leptos::server::Resource::new_bitcode_blocking(
            move || summoner_identifier_memo.get(),
            |summoner_identifier| async move {
                get_summoner(
                    summoner_identifier
                )
                .await
            },
        );
        Self { summoner_resource, summoner_identifier_memo }
    }

    fn view(this: Self) -> AnyView {
        let SummonerPageRoute { summoner_resource,summoner_identifier_memo } = this;
        provide_context(summoner_identifier_memo);
        let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
        let location = use_location();
        view! {
            <div class="my-0 mx-auto max-w-5xl text-center">
                <A href="/" attr:class="p-6 text-4xl my-4">
                    "Welcome to Ruche"
                </A>
                <SummonerSearch is_summoner_page=true />

                <Transition fallback=move || {
                    view! { <div class="text-center">Loading Summoner</div> }
                }>
                    {move || Suspend::new(async move {
                        match summoner_resource.await {
                            Ok(summoner) => {
                                Either::Left({
                                    let (level_signal, set_level) = signal(summoner.summoner_level);
                                    let (profile_icon_signal, set_profile_icon) = signal(
                                        summoner.profile_icon_id,
                                    );
                                    let (summoner, _) = signal(summoner);
                                    provide_context(summoner);
                                    let update_summoner_action = ServerAction::<
                                        UpdateSummoner,
                                    >::new();
                                    let pending = RwSignal::new(false);
                                    Effect::new(move |_| {
                                        let _ = update_summoner_action.version().get();
                                        pending.set(false);
                                        if let Some(Ok(Some((level, profile_icon_id)))) = update_summoner_action
                                            .value()
                                            .get()
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
                                     {
                                        use futures::StreamExt;
                                        use send_wrapper::SendWrapper;
                                        use crate::utils::SSEEvent;
                                        let mut source = SendWrapper::new(
                                            gloo_net::eventsource::futures::EventSource::new(
                                                    format!(
                                                        "/sse/match_updated/{}/{}",
                                                        summoner.read().platform,
                                                        summoner.read().id,
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
                                        let sse_match_update_version = expect_context::<RwSignal<Option<SSEMatchUpdateVersion>>>();
                                        let sse_in_live = expect_context::<RwSignal<SSEInLiveGame>>();
                                        Effect::new(move |_| {
                                            let event = s.get();
                                            match event {
                                                Some(SSEEvent::SummonerMatches(version)) => {
                                                    sse_match_update_version
                                                        .set(Some(SSEMatchUpdateVersion(version)));
                                                }
                                                Some(SSEEvent::LiveGame(version)) => {
                                                    sse_in_live.set(SSEInLiveGame(version));
                                                }
                                                _ => {}
                                            }
                                        });
                                    }
                                    meta_store
                                        .image()
                                        .set(
                                            ProfileIcon(summoner.read().profile_icon_id)
                                                .get_static_asset_url(),
                                        );

                                    view! {
                                        <div class="flex justify-center">
                                            <div class="flex w-[768px] my-2 space-x-2">
                                                <SummonerInfo
                                                    game_name=summoner.read().game_name.clone()
                                                    tag_line=summoner.read().tag_line.clone()
                                                    pro_slug=summoner.read().pro_slug
                                                    platform=summoner.read().platform
                                                    level_signal=level_signal
                                                    profile_icon_signal=profile_icon_signal
                                                />
                                                <div class="h-fit">

                                                    <button
                                                        class="my-button flex items-center"
                                                        on:click=move |e| {
                                                            e.prevent_default();
                                                            pending.set(true);
                                                            update_summoner_action
                                                                .dispatch(UpdateSummoner {
                                                                    summoner_id: summoner.read().id,
                                                                    game_name: summoner.read().game_name.clone(),
                                                                    tag_line: summoner.read().tag_line.clone(),
                                                                    platform_route: summoner.read().platform,
                                                                });
                                                        }
                                                    >
                                                        <PendingLoading pending>Update</PendingLoading>
                                                    </button>

                                                </div>
                                            </div>
                                        </div>
                                    }
                                })
                            }
                            Err(_) => Either::Right(()),
                        }
                    })}
                </Transition>
                <SummonerNav />
                <MatchFilters hidden=Signal::derive(move || {
                    location.pathname.get().ends_with("/live")
                })>
                    <Outlet />
                </MatchFilters>
            </div>
        }.into_any()
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
                <A href=summoner_url(
                    platform.code(),
                    game_name.as_ref(),
                    tag_line.as_ref(),
                )>{game_name.clone()}#{tag_line.clone()}</A>
                <div class="flex ">
                    <span>lvl. {move || level_signal()}</span>
                    <ProPlayerSlugView pro_player_slug=pro_slug small=false />
                </div>
            </div>
        </div>
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Encode, Decode)]
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
            self.platform.code(),
            self.game_name.as_ref(),
            self.tag_line.as_ref(),
        )
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub struct SSEMatchUpdateVersion(pub u16);

#[derive(Clone, PartialEq, Eq, Copy, Default)]
pub struct SSEInLiveGame(pub Option<u16>);
