use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_champions::get_champions;
use crate::consts::Champion;
use crate::views::summoner_page::Summoner;
use crate::views::MatchFiltersSearch;
use itertools::Itertools;
use leptos::either::Either;
use leptos::prelude::{expect_context, signal, Children, ClassAttribute, ElementChild, For, Get, OnAttribute, ReadSignal, Resource, RwSignal, Set, Suspend, Suspense, WriteSignal};
use leptos::{component, view, IntoView};
use serde::{Deserialize, Serialize};


#[component]
pub fn SummonerChampionsPage() -> impl IntoView {
    let summoner = expect_context::<ReadSignal<Summoner>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<MatchFiltersSearch>>();
    let (table_sort, set_table_sort) = signal::<(TableSortType, bool)>((TableSortType::default(), true));

    let champions_resource = Resource::new(
        move || (match_filters_updated.get(), summoner()),
        |(filters, summoner)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_champions(summoner.id, Some(filters)).await
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
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::Index
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("text-center")
                                                            >
                                                                #
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::Champion
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Champion
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::WinRate
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Win Rate
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgKDA
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg KDA
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgGold
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg Gold
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgCs
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg Cs
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgDamageDealt
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Avg Damage Dealt
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgDamageTaken
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Avg Damage Taken
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::DoubleKills
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Double kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::TripleKills
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Triple kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::QuadraKills
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Quadra kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 h-full">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::PentaKills
                                                                table_sort=table_sort.clone()
                                                                set_table_sort=set_table_sort.clone()
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Penta kills
                                                            </TableHeaderItem>
                                                        </th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    <For
                                                        each=move || {
                                                            champions
                                                                .clone()
                                                                .into_iter()
                                                                .enumerate()
                                                                .sorted_by(|(idx_a, a), (idx_b, b)| {
                                                                    let (sort_type, reverse) = table_sort.get();
                                                                    sort_type.sort(idx_a, a, idx_b, b, reverse)
                                                                })
                                                        }
                                                        key=|(id, champion)| champion.champion_id
                                                        let:champion_with_index
                                                    >
                                                        {
                                                            let (index, champion): (usize, ChampionStats) = champion_with_index;
                                                            view! {
                                                                <tr class="p-1">
                                                                    <td class="text-center bg-gray-800 border border-gray-700">
                                                                        {index + 1}
                                                                    </td>
                                                                    <td class="text-left border border-gray-800">
                                                                        <div class="flex items-center">
                                                                            <div class="py-1">
                                                                                <img
                                                                                    src=Champion::get_static_url(champion.champion_id)
                                                                                    alt=format!(
                                                                                        "Champion {}",
                                                                                        Champion::try_from(champion.champion_id as i16)
                                                                                            .unwrap()
                                                                                            .to_string(),
                                                                                    )
                                                                                    class="w-[32px] h-[32px] rounded-full"
                                                                                    width="32"
                                                                                    height="32"
                                                                                />
                                                                            </div>
                                                                            <div class="ml-2 text-center">{champion.champion_name}</div>
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
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.avg_gold_earned.round()}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.avg_cs.round()}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.avg_damage_dealt_to_champions.round()}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.avg_damage_taken.round()}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.total_double_kills}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.total_triple_kills}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.total_quadra_kills}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.total_penta_kills}
                                                                    </td>
                                                                </tr>
                                                            }
                                                        }

                                                    </For>

                                                </tbody>
                                            </table>
                                        }
                                    }),
                                )
                            } else {
                                Ok(
                                    Either::Right(

                                        view! { <p class="my-2">No Champions found</p> },
                                    ),
                                )
                            }
                        }
                        Err(e) => Err(e),
                    }
                })}
            </Suspense>
        </div>
    }
}


#[component]
pub fn TableHeaderItem(
    sort_type: TableSortType,
    table_sort: ReadSignal<(TableSortType, bool)>,
    set_table_sort: WriteSignal<(TableSortType, bool)>,
    #[prop(optional)]
    class: Option<String>,
    children: Children,
) -> impl IntoView {
    let is_active = move || table_sort.get().0 == sort_type;
    let is_reverse = move || table_sort.get().1;
    let toggle_sort = move |sort_type: TableSortType| {
        let (sort, reverse) = table_sort.get();
        if sort == sort_type {
            set_table_sort((sort_type, !reverse));
        } else {
            set_table_sort((sort_type, true));
        }
    };
    view! {
        <button
            class=("border-t-2", move || is_active() && is_reverse())
            class=("border-b-2", move || is_active() && !is_reverse())
            class=format!("h-full w-full border-blue-500 {}", class.unwrap_or_default())
            on:click=move |_| toggle_sort(sort_type)
        >
            {children()}
        </button>
    }
}


#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum TableSortType {
    #[default]
    Index,
    Champion,
    WinRate,
    AvgKDA,
    AvgGold,
    AvgCs,
    AvgDamageDealt,
    AvgDamageTaken,
    DoubleKills,
    TripleKills,
    QuadraKills,
    PentaKills,
}

impl TableSortType {
    pub fn sort(&self, idx_a: &usize, a: &ChampionStats, idx_b: &usize, b: &ChampionStats, reverse: bool) -> std::cmp::Ordering {
        let ordering = match self {
            TableSortType::Index => idx_b.cmp(idx_a),
            TableSortType::Champion => b.champion_name.cmp(&a.champion_name),
            TableSortType::WinRate => a.win_rate.partial_cmp(&b.win_rate).unwrap(),
            TableSortType::AvgKDA => a.avg_kda.partial_cmp(&b.avg_kda).unwrap(),
            TableSortType::AvgGold => a.avg_gold_earned.partial_cmp(&b.avg_gold_earned).unwrap(),
            TableSortType::AvgCs => a.avg_cs.partial_cmp(&b.avg_cs).unwrap(),
            TableSortType::AvgDamageDealt => a.avg_damage_dealt_to_champions.partial_cmp(&b.avg_damage_dealt_to_champions).unwrap(),
            TableSortType::AvgDamageTaken => a.avg_damage_taken.partial_cmp(&b.avg_damage_taken).unwrap(),
            TableSortType::DoubleKills => a.total_double_kills.cmp(&b.total_double_kills),
            TableSortType::TripleKills => a.total_triple_kills.cmp(&b.total_triple_kills),
            TableSortType::QuadraKills => a.total_quadra_kills.cmp(&b.total_quadra_kills),
            TableSortType::PentaKills => a.total_penta_kills.cmp(&b.total_penta_kills),
        };
        if reverse {
            ordering.reverse()
        } else {
            ordering
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChampionStats {
    pub champion_id: i32,
    pub champion_name: String,
    pub total_matches: i64,
    pub total_wins: i64,
    pub total_lose: i64,
    pub win_rate: f64,
    pub avg_kda: f64,
    pub avg_kill_participation: f64,
    pub avg_kills: f64,
    pub avg_deaths: f64,
    pub avg_assists: f64,
    pub avg_gold_earned: f64,
    pub avg_cs: f64,
    pub avg_damage_dealt_to_champions: f64,
    pub avg_damage_taken: f64,
    pub total_double_kills: i64,
    pub total_triple_kills: i64,
    pub total_quadra_kills: i64,
    pub total_penta_kills: i64,
}