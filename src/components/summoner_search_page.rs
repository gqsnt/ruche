use std::str::FromStr;
use leptos::{component, create_effect, create_server_action, create_signal, expect_context, server, use_context, view, CollectView, IntoView, RwSignal, ServerFnError, SignalGet, SignalWith};
use leptos::html::A;
use leptos_router::{use_params_map, use_query_map, ActionForm, A};
#[cfg(feature = "ssr")]
use sqlx::query;
#[cfg(feature = "ssr")]
use crate::AppState;
use crate::models::entities::summoner::Summoner;
use crate::models::types::PlatformType;

#[component]
pub fn SummonerSearchPage() -> impl IntoView {
    let query = use_query_map();
    let params = use_params_map();
    let find_summoner = create_server_action::<FindSummoner>();
    let platform_type = use_context::<RwSignal<PlatformType>>().expect("PlatformType signal not found");

    let game_name = move || {
        query.with(|query| query.get("game_name").cloned().unwrap_or_default())
    };
    let tag_line = move || {
        query.with(|query| query.get("tag_line").cloned().unwrap_or_default())
    };




    view! {
        <ActionForm action=find_summoner>
                <input type="text" placeholder="Game Name" prop:value=game_name() name="game_name" />
                <input type="text" placeholder="Tag Line" prop:value=tag_line()  name="tag_line"/>
                <select   name="platform_type"  prop:value=move ||platform_type.read_only().get().to_string()>
                    {PlatformType::cases().into_iter()
                        .map(|pt| view! {
                            <option
                        value=format!("{pt}")
                        selected = pt == &PlatformType::EUW1
                        >{format!("{pt}")}</option>
                        })
                        .collect_view()
                    }
                </select>
        <button type="submit">"Search"</button>
        </ActionForm>
    }
}



#[server(FindSummoner, "/api")]
pub async fn find_summoner(
    game_name: String,
    tag_line: String,
    platform_type: String,
) -> Result<(), ServerFnError> {
    let state = expect_context::<AppState>();
    let db = state.db.clone();
    let platform_type = PlatformType::from_code(platform_type.as_str()).unwrap();

    match Summoner::find_by_details(&db, &platform_type, game_name.as_str(), tag_line.as_str()).await {
        Ok(summoner) => {
            // Generate slug for URL
            let slug = summoner.slug();
            let url = format!(
                "/{}/summoners/{}/matches",
                platform_type.to_string(),
                slug,
            );
            leptos_axum::redirect(url.as_str());
        }
        Err(_) => {
            let not_found_url = format!(
                "/{}?game_name={}&tag_line={}",
                platform_type.to_string(),
                game_name,
                tag_line
            );
            let riot_api = state.riot_api.clone();
            let region = platform_type.region().to_riven();
            match riot_api
                .account_v1()
                .get_by_riot_id(region, game_name.as_str(), tag_line.as_str())
                .await
            {
                Ok(Some(account)) => {
                    let riven_route = platform_type.to_riven();
                    match riot_api
                        .summoner_v4()
                        .get_by_puuid(riven_route, account.puuid.as_str())
                        .await
                    {
                        Ok(summoner_data) => {
                            let slug = Summoner::generate_slug(&account.game_name.clone().unwrap(), &account.tag_line.clone().unwrap());
                            let id = Summoner::insert_or_update_account_and_summoner(
                                &db,
                                platform_type,
                                account,
                                summoner_data,
                            )
                                .await?;
                            // Generate slug for URL

                            let url = format!(
                                "/{}/summoners/{}/matches",
                                platform_type.to_string(),
                                slug,
                            );
                            leptos_axum::redirect(url.as_str());
                        }
                        _ => {
                            leptos_axum::redirect(not_found_url.as_str());
                        }
                    }
                }
                _ => {
                    leptos_axum::redirect(not_found_url.as_str());
                }
            }
        }
    }
    Ok(())
}