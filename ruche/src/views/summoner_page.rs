use crate::app::{
    to_summoner_identifier_memo, MetaStore, MetaStoreStoreFields, SummonerIdentifier,
    SummonerRouteParams,
};
use crate::backend::server_fns::get_summoner::get_summoner;
use crate::backend::server_fns::update_summoner::UpdateSummoner;
use crate::utils::{summoner_url, ProPlayerSlug};
use crate::views::components::match_filters::MatchFilters;
use crate::views::summoner_page::summoner_nav::SummonerNav;
use crate::views::{BackEndMatchFiltersSearch, ImgSrc, PendingLoading, ProPlayerSlugView};
use bitcode::{Decode, Encode};
use common::consts::platform_route::PlatformRoute;
use common::consts::profile_icon::ProfileIcon;
use common::consts::HasStaticSrcAsset;
use leptos::context::provide_context;
use leptos::either::Either;

use leptos::prelude::*;
use leptos::server::codee::binary::BitcodeCodec;
use leptos::{component, view, IntoView};

use crate::views::summoner_page::summoner_search_page::SummonerSearch;
use leptos_router::components::{Outlet, A};
use leptos_router::hooks::{use_location, use_params};
use leptos_router::{lazy_route, LazyRoute};
use reactive_stores::Store;

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
    pub summoner_identifier_memo: Memo<SummonerIdentifier>,
}

pub fn expect_filters() -> Store<BackEndMatchFiltersSearch>{
    #[cfg(feature = "ssr")]
    {
        Store::new(BackEndMatchFiltersSearch::default())
    }
    #[cfg(not(feature = "ssr"))]
    {
        expect_context()
    }
}


#[lazy_route]
impl LazyRoute for SummonerPageRoute {
    fn data() -> Self {
        let summoner_route_params = use_params::<SummonerRouteParams>();
        let summoner_identifier_memo = to_summoner_identifier_memo(summoner_route_params);

        let summoner_resource = leptos::server::Resource::new_bitcode_blocking(
            move || summoner_identifier_memo.get(),
            |summoner_identifier| async move { get_summoner(summoner_identifier).await },
        );
        Self {
            summoner_resource,
            summoner_identifier_memo,
        }
    }

    fn view(this: Self) -> AnyView {
        let SummonerPageRoute {
            summoner_resource,
            summoner_identifier_memo,
        } = this;
        provide_context(summoner_identifier_memo);

        let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
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
                                        use reactive_stores::Store;
                                        use futures::StreamExt;
                                        use send_wrapper::SendWrapper;
                                        use crate::utils::SSEVersions;
                                        let summoner_sse_url = format!(
                                            "/sse/match_updated/{}/{}",
                                            summoner.platform,
                                            summoner.id,
                                        );
                                        let mut source = SendWrapper::new(
                                            gloo_net::eventsource::futures::EventSource::new(
                                                    &summoner_sse_url,
                                                )
                                                .expect("couldn't connect to SSE stream"),
                                        );
                                        let s = ReadSignal::from_stream_unsync(
                                            source
                                                .subscribe("message")
                                                .expect("couldn't subscribe to SSE stream")
                                                .filter_map(|value| async move {
                                                    value
                                                        .ok()
                                                        .and_then(|(_, ev)| ev.data().as_string())
                                                        .and_then(|s| SSEVersions::parse(&s))
                                                }),
                                        );
                                        on_cleanup(move || source.take().close());
                                        let sse_versions = expect_context::<Store<SSEVersions>>();
                                        Effect::watch(
                                            move || s.get(),
                                            move |new_ver, _, _| {
                                                if let Some(ver) = *new_ver {
                                                    let prev = sse_versions.get_untracked();
                                                    if prev != ver {
                                                        sse_versions.set(ver);
                                                    }
                                                }
                                            },
                                            false,
                                        );
                                    }
                                    meta_store
                                        .image()
                                        .set(
                                            ProfileIcon(summoner.profile_icon_id).get_static_asset_url(),
                                        );

                                    // Mise à jour seulement si différent

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
                                                            pending.set(true);
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
                                    }
                                })
                            }
                            Err(_) => Either::Right(()),
                        }
                    })}
                </Transition>
                <SummonerNav />
               
                  <Outlet />

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
