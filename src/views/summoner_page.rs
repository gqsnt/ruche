use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_summoner::get_summoner;
use crate::backend::server_fns::update_summoner::UpdateSummoner;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::profile_icon::ProfileIcon;
use crate::consts::HasStaticAsset;
use crate::utils::{summoner_url, GameName, ProPlayerSlug, SummonerSlug, TagLine};
use crate::views::summoner_page::summoner_nav::SummonerNav;
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::Read;
use leptos::prelude::*;
use leptos::server_fn::rkyv::{Archive, Deserialize, Serialize};
use leptos::{component, view, IntoView};
use leptos_router::hooks::use_params_map;

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
    let summoner_resource = leptos_server::Resource::new_rkyv(
        move || (platform_type(), summoner_slug()),
        |(platform, summoner_slug)| async move {
            get_summoner(
                PlatformRoute::from(platform.as_str()),
                SummonerSlug::new(summoner_slug.as_str()),
            )
            .await
        },
    );

    let summoner_view = Suspend::new(async move {
        match summoner_resource.await {
            Ok(summoner) => Either::Left({
                let (level_signal, set_level) = signal(summoner.summoner_level);
                let (profile_icon_signal, set_profile_icon) = signal(summoner.profile_icon_id);
                provide_context(summoner);
                let update_summoner_action = ServerAction::<UpdateSummoner>::new();
                Effect::new(move |_| {
                    let _ = update_summoner_action.version().get();
                    if let Some(Ok(Some(summoner_update))) =
                        update_summoner_action.value().read_only().get()
                    {
                        if summoner_update.summoner_level != level_signal() {
                            set_level(summoner_update.summoner_level);
                        }
                        if summoner_update.profile_icon_id != profile_icon_signal() {
                            set_profile_icon(summoner_update.profile_icon_id);
                        }
                    }
                });
                #[cfg(not(feature = "ssr"))]
                let summoner_update_version = {
                    use futures::StreamExt;
                    use send_wrapper::SendWrapper;
                    let mut source = SendWrapper::new(
                        gloo_net::eventsource::futures::EventSource::new(
                            format!("/sse/match_updated/{}", summoner.id,).as_str(),
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
                                        message_event
                                            .data()
                                            .as_string()
                                            .expect("failed to parse sse string")
                                            .parse::<u16>()
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
                let (summoner_update_version, _) = signal(None::<u16>);
                provide_context(summoner_update_version);
                meta_store
                    .image()
                    .set(ProfileIcon(summoner.profile_icon_id).get_static_asset_url());

                view! {
                    {move || {
                        view! {
                            <div class="flex justify-center">
                                <div class="flex justify-between w-[768px] mb-2">
                                    <div class="flex  mt-2 space-x-2">
                                        {move || {
                                            let profile_icon = ProfileIcon(profile_icon_signal());
                                            view! {
                                                <img
                                                    alt=profile_icon.to_string()
                                                    src=profile_icon.get_static_asset_url()
                                                    class="w-16 h-16"
                                                />
                                            }
                                        }}
                                        <div class="flex flex-col items-start">
                                            <div>
                                                {summoner.game_name.to_string()}#
                                                {summoner.tag_line.to_string()}
                                            </div>
                                            <div>
                                                <span>lvl. {move || level_signal()}</span>
                                                <Show when=move || { summoner.pro_slug.is_some() }>

                                                    <a
                                                        target="_blank"
                                                        href=format!(
                                                            "https://lolpros.gg/player/{}",
                                                            summoner.pro_slug.unwrap().to_string(),
                                                        )
                                                        class=" bg-purple-800 rounded px-1 py-0.5 text-center ml-1"
                                                    >
                                                        PRO
                                                    </a>
                                                </Show>

                                            </div>
                                        </div> <div>
                                            <button
                                                class="my-button"
                                                on:click=move |e| {
                                                    e.prevent_default();
                                                    update_summoner_action
                                                        .dispatch(UpdateSummoner {
                                                            summoner_id: summoner.id,
                                                            game_name: summoner.game_name,
                                                            tag_line: summoner.tag_line,
                                                            platform_route: summoner.platform,
                                                        });
                                                }
                                            >
                                                Update
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }}

                    <SummonerNav summoner />
                }
            }),
            Err(_) => Either::Right(()),
        }
    });

    view! {
        <Transition fallback=move || {
            view! { <div class="text-center">Loading Summoner</div> }
        }>{summoner_view}</Transition>
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Copy, Serialize, Deserialize, Archive)]
pub struct Summoner {
    pub id: i32,
    pub game_name: GameName,
    pub pro_slug: Option<ProPlayerSlug>,
    pub tag_line: TagLine,
    pub profile_icon_id: u16,
    pub summoner_level: u16,
    pub platform: PlatformRoute,
}

impl Summoner {
    pub fn to_route_path(&self) -> String {
        summoner_url(
            self.platform.to_string(),
            self.game_name.to_string(),
            self.tag_line.to_string(),
        )
    }
}
