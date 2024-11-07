use crate::apis::{get_summoner_champions, MatchFiltersSearch};
use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::consts::Champion;
use crate::models::entities::summoner::Summoner;
use leptos::either::Either;
use leptos::prelude::{expect_context, ClassAttribute, ElementChild, Get, ReadSignal, Resource, RwSignal, Set, Suspend, Suspense};
use leptos::{component, view, IntoView};

#[component]
pub fn SummonerChampionsPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<MatchFiltersSearch>>();

    let champions_resource = Resource::new(
        move || (match_filters_updated.get(), summoner()),
        |(filters, summoner)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_summoner_champions(summoner.id, Some(filters)).await
        },
    );

    meta_store.title().set(format!("{}#{} | Champions | Broken.gg", summoner().game_name, summoner().tag_line));
    meta_store.description().set(format!("Discover the top champions played by {}#{} on League Of Legends. Access in-depth statistics, win rates, and performance insights on Broken.gg, powered by Rust for optimal performance.", summoner().game_name, summoner().tag_line));
    meta_store.url().set(format!("{}?tab=champions", summoner().to_route_path()));
    view! {
        <div>
            <Suspense fallback=move || {
                view! { <p>Loading Champions ...</p> }
            }>
                {move || Suspend::new(async move {
                    match champions_resource.await {
                        Ok(champions) => {
                            if !champions.is_empty() {
                                Ok(
                                    Either::Left({
                                        view! {
                                            <table class="w-full table-fixed bg-gray-700 border-collapse  my-2 border border-gray-600">
                                                <colgroup>
                                                    <col width="45" />
                                                    <col width="auto" />
                                                    <col width="140" />
                                                    <col width="105" />
                                                    <col width="88" />
                                                    <col width="72" />
                                                    <col width="66" />
                                                    <col width="66" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                </colgroup>
                                                <thead>
                                                    <tr class="bg-gray-800 text-sm h-[32px]">
                                                        <th class="border border-gray-700 text-center">#</th>
                                                        <th class="border border-gray-700 text-left"><div class="ml-2">Champion</div></th>
                                                        <th class="border border-gray-700 ">Played</th>
                                                        <th class="border border-gray-700 ">Avg KDA</th>
                                                        <th class="border border-gray-700 ">Avg Gold</th>
                                                        <th class="border border-gray-700 ">Avg Cs</th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Avg Damage Dealt</div></th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Avg Damage Taken</div></th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Double kills</div></th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Triple kills</div></th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Quadra kills</div></th>
                                                        <th class="border border-gray-700"><div class="text-ellipsis whitespace-nowrap overflow-hidden">Penta kills</div></th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {champions
                                                        .into_iter()
                                                        .enumerate()
                                                        .map(|(index, champion)| {
                                                            view! {
                                                                <tr class="p-1">
                                                                    <td class="text-center bg-gray-800 border border-gray-800">{index + 1}</td>
                                                                    <td class="text-left border border-gray-800">
                                                                        <div class="flex items-center">
                                                                            <div class="py-1">
                                                                                <img
                                                                                    src=Champion::get_static_url(champion.champion_id)
                                                                                    alt=Champion::try_from(champion.champion_id as i16)
                                                                                        .unwrap()
                                                                                        .to_string()
                                                                                    class="w-[32px] h-[32px] rounded-full"
                                                                                    width="32"
                                                                                    height="32"/>
                                                                            </div>
                                                                            <div class="ml-2 text-center">
                                                                                {Champion::try_from(champion.champion_id as i16)
                                                                                    .unwrap()
                                                                                    .to_string()}
                                                                            </div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="text-xs border border-gray-800">
                                                                        {champion.total_wins}W {champion.total_lose}L
                                                                        {champion.win_rate}%
                                                                    </td>
                                                                    <td class="text-xs border border-gray-800">
                                                                        <div>
                                                                            <div>{champion.avg_kda}:1</div>
                                                                            <div>
                                                                                {champion.avg_kills}/ {champion.avg_deaths}/
                                                                                {champion.avg_assists}
                                                                            </div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">{champion.avg_gold_earned.round()}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.avg_cs.round()}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.avg_damage_dealt_to_champions.round()}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.avg_damage_taken.round()}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.total_double_kills}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.total_triple_kills}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.total_quadra_kills}</td>
                                                                    <td class="border border-gray-800 text-xs">{champion.total_penta_kills}</td>
                                                                </tr>
                                                            }
                                                        })
                                                        .collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        }
                                    }),
                                )
                            } else {
                                Ok(Either::Right(view! { <p class="my-2">No Champions found</p> }))
                            }
                        }
                        Err(e) => Err(e),
                    }
                })}
            </Suspense>
        </div>
    }
}