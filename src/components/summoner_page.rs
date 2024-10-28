use std::str::FromStr;
use leptos::{component, create_blocking_resource, create_effect, create_isomorphic_effect, create_server_action, create_signal, expect_context, server, spawn_local, use_context, view, IntoView, RwSignal, ServerFnError, SignalGet, SignalSet, SignalWith, Suspense, Transition};
use leptos_router::{use_location, use_navigate, use_params_map, ActionForm, Outlet};
#[cfg(feature = "ssr")]
use crate::AppState;
use crate::models::entities::summoner::Summoner;
use crate::models::types::PlatformType;
#[cfg(feature = "ssr")]
use crate::models::update::summoner_matches::update_summoner_matches;

#[server(GetSummoner, "/api")]
pub async fn get_summoner(
    platform_type: String,
    summoner_slug: String,
) -> Result<Summoner, ServerFnError> {
    println!("SERVER: get_summoner {} {}", platform_type, summoner_slug);
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let platform_type = PlatformType::from_code(platform_type.as_str()).unwrap();
    let (game_name, tag_line) = Summoner::parse_slug(summoner_slug.as_str()).unwrap();
    if let Ok(summoner) = Summoner::find_by_details(&db, &platform_type, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner)
    } else {
        let (game_name, tag_line) = Summoner::parse_slug(summoner_slug.as_str()).unwrap();
        leptos_axum::redirect(format!("/{}?game_name={}&tag_line={}", platform_type.to_string(), game_name, tag_line).as_str());
        Err(ServerFnError::new("Summoner not found"))
    }
}

#[server(UpdateSummoner, "/api")]
pub async fn update_summoner(id:String, puuid: String, platform_type: String) -> Result<(), ServerFnError>{
    let platform_type = PlatformType::from_code(platform_type.as_str()).unwrap();
    let state = expect_context::<AppState>();
    let riot_api = state.riot_api.clone();
    let region = platform_type.region().to_riven();
    match riot_api.account_v1()
        .get_by_puuid(region, puuid.as_str())
        .await{
        Ok(account) => {
            let riven_route = platform_type.to_riven();
            match riot_api.summoner_v4().get_by_puuid(riven_route, account.puuid.as_str()).await{
                Ok(summoner) => {
                    let db = state.db.clone();
                    let puuid = summoner.puuid.clone();
                    let slug = Summoner::generate_slug(&account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                    leptos_axum::redirect(format!("/{}/summoners/{}/matches", platform_type.to_string(), slug).as_str());
                    Summoner::insert_or_update_account_and_summoner(&db, platform_type, account, summoner).await?;
                    tokio::spawn(async move {
                        let _ = update_summoner_matches(db.clone(),riot_api,puuid,platform_type, 1000).await;
                        println!("Matches updated");
                    });
                }
                _ => {
                }
            }
        }
        Err(_) => {
        }
    }
    Ok(())
}

#[component]
pub fn SummonerPage() -> impl IntoView {
    let navigate = use_navigate();
    let params = use_params_map();
    let location = use_location();

    let update_summoner_action = create_server_action::<UpdateSummoner>();

    let platform_type = use_context::<RwSignal<PlatformType>>().expect("PlatformType signal not found");
    let summoner_slug = move || {
        params.with(|params| params.get("summoner_slug").cloned().unwrap())
    };

    let summoner = create_blocking_resource(
        move ||(platform_type.get().to_string(), summoner_slug(), update_summoner_action.version().get()),
        move |(platform_type, summoner_slug,_ )| async move {
            update_summoner_action.value().set(None);
            println!("CLIENT: get_summoner {} {}", platform_type, summoner_slug);
            get_summoner(platform_type, summoner_slug).await
        },
    );

    view! {
        <Transition fallback=move || view!{<div>"Loading..."</div>}>
            {move || {
                match summoner.get() {
                    Some(Ok(summoner)) => {
                        let summoner_id = use_context::<RwSignal<Option<i32>>>().expect("summoner_id not found");
                        let summoner_route = format!(
                            "/{}/summoners/{}",
                            summoner.platform,
                            summoner.slug(),
                        );
                        summoner_id.set(Some(summoner.id));
                        let summoner_id_ = summoner.id;
                        let puuid_ = summoner.puuid.clone();
                        let platform_type_ = platform_type.get();
                        let is_active = |route: &str| {
                            location.pathname.get().contains(route)
                        };

                        view! {
                            <div class="flex justify-between">
                                <div class="flex justify-center items-center mt-2">
                                    <img src=summoner.profile_icon_url() class="w-16 h-16"/>
                                    <div class="flex flex-col items-start">
                                        <div>{summoner.game_name} #{summoner.tag_line}</div>
                                        <div >lvl. {summoner.summoner_level}</div>
                                    </div>
                                    <ActionForm action=update_summoner_action >
                                        <input type="hidden" name="id" value=move || summoner.id/>
                                        <input type="hidden" name="puuid" value=move || summoner.puuid.clone()/>
                                        <input type="hidden" name="platform_type" value=move || platform_type.get().to_string()/>
                                        <button class="ml-2 bg-green-500 px-3 py-1" type="submit">Update</button>
                                    </ActionForm>
                                </div>
                            </div>
                            <nav>
                                <ul class="flex border-b">
                                  <li class="-mb-px mr-1">
                                    <a class=if is_active("matches") { "active-tab" } else { "default-tab" } href=format!("{}/matches",summoner_route)>Matches</a>
                                  </li>
                                  <li class="-mb-px mr-1">
                                    <a class=if is_active("champions") { "active-tab" } else { "default-tab" } href=format!("{}/champions",summoner_route)>Champions</a>
                                  </li>
                                  <li class="-mb-px mr-1">
                                    <a class=if is_active("encounters") { "active-tab" } else { "default-tab" } href=format!("{}/encounters",summoner_route)>Encounters</a>
                                  </li>
                                  <li class="-mb-px mr-1">
                                    <a class=if is_active("live") { "active-tab" } else { "default-tab" } href=format!("{}/live",summoner_route)>Live</a>
                                  </li>
                                </ul>
                            </nav>
                            <Outlet/>
                        }.into_view()
                    }
                    _ => ().into_view(),
                }
            }}
        </Transition>
    }
}