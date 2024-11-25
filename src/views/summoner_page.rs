use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_summoner::get_summoner;
use crate::backend::server_fns::update_summoner::UpdateSummoner;
use crate::consts::platform_route::PlatformRoute;
use crate::consts::profile_icon::ProfileIcon;
use crate::consts::HasStaticAsset;
use crate::utils::{summoner_url, GameName, ProPlayerSlug, Puuid, SummonerSlug, TagLine};
use crate::views::summoner_page::summoner_nav::SummonerNav;
use leptos::context::provide_context;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::{component, view, IntoView};
use leptos::prelude::Read;
use leptos_router::hooks::use_params_map;
use leptos::server_fn::rkyv::{Deserialize, Serialize, Archive};


pub mod summoner_search_page;
pub mod summoner_matches_page;
pub mod summoner_champions_page;
pub mod summoner_encounters_page;
pub mod summoner_live_page;
pub mod match_details;
pub mod summoner_nav;
pub mod summoner_encounter_page;

#[component]
pub fn SummonerPage() -> impl IntoView {
    let update_summoner_action = ServerAction::<UpdateSummoner>::new();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let params = use_params_map();

    let platform_type = move || {
        params.read().get("platform_type").clone().unwrap_or_default()
    };
    let summoner_slug = move || {
        params.read().get("summoner_slug").clone().unwrap_or_default()
    };


    // Update the summoner signal when resource changes
    let summoner_resource = leptos_server::Resource::new_rkyv_blocking(
        move || (update_summoner_action.version().get(), platform_type(), summoner_slug()),
        |(_, platform, summoner_slug)| async move {
            //log!("Client::Fetching summoner: {}", ss);
            get_summoner(PlatformRoute::from(platform.as_str()), SummonerSlug::new(summoner_slug.as_str())).await
        },
    );



    let summoner_view = move || {
        Suspend::new(async move {
            match summoner_resource.await {
                Ok(summoner) => {
                    Either::Left({
                        let (summoner_signal, _) = signal(summoner.clone());
                        provide_context(summoner_signal);
                        provide_context(update_summoner_action.version());
                        meta_store.image().set(ProfileIcon::get_static_asset_url(summoner.profile_icon_id));
                        view! {
                            <div class="flex justify-center">
                                <div class="flex justify-between w-[768px] mb-2">
                                    <div class="flex  mt-2 space-x-2">
                                        <img
                                            alt="Profile Icon"
                                            src=ProfileIcon::get_static_asset_url(
                                                summoner.profile_icon_id,
                                            )
                                            class="w-16 h-16"
                                        />
                                        <div class="flex flex-col items-start">
                                            <div>
                                                {summoner_signal().game_name.to_string()}#
                                                {summoner_signal().tag_line.to_string()}
                                            </div>
                                            <div>
                                                <span>lvl. {summoner.summoner_level}</span>
                                                <Show when=move || summoner_signal().pro_slug.is_some()>

                                                    <a
                                                        target="_blank"
                                                        href=format!(
                                                            "https://lolpros.gg/player/{}",
                                                            summoner_signal().pro_slug.as_ref().unwrap().to_string(),
                                                        )
                                                        class=" bg-purple-800 rounded px-1 py-0.5 text-center ml-1"
                                                    >
                                                        PRO
                                                    </a>
                                                </Show>

                                            </div>
                                        </div>
                                        <div>
                                            <button
                                                class="my-button"
                                                on:click=move |_| {
                                                    update_summoner_action
                                                        .dispatch(UpdateSummoner {
                                                            puuid: summoner_signal().puuid,
                                                            platform_route: summoner_signal().platform,
                                                        });
                                                }
                                            >
                                                Update
                                            </button>
                                        </div>
                                    </div>
                                </div>

                            </div>
                            <SummonerNav />
                        }
                    })
                }
                Err(_) => { Either::Right(()) }
            }
        })
    };

    view! {
        <Transition fallback=move || {
            view! { <div class="text-center">Loading Summoner</div> }
        }>{summoner_view}</Transition>
    }
}



#[derive(Clone, PartialEq, Eq,Debug, Serialize, Deserialize, Archive)]
pub struct Summoner {
    pub id: i32,
    pub profile_icon_id: u16,
    pub summoner_level: u16,
    pub platform: PlatformRoute,
    pub game_name: GameName,
    pub tag_line: TagLine,
    pub puuid: Puuid,
    pub pro_slug: Option<ProPlayerSlug>
}


impl Summoner {
    pub fn to_route_path(&self) -> String {
        summoner_url(self.platform.to_string(), self.game_name.to_string(), self.tag_line.to_string())
    }
}

