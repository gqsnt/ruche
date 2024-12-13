use crate::app::{MetaStore, MetaStoreStoreFields};
use crate::backend::server_fns::get_champions::get_champions;
use crate::utils::{calculate_and_format_kda, format_float_to_2digits, format_with_spaces};
use crate::views::summoner_page::{SSEMatchUpdateVersion, Summoner};
use crate::views::{BackEndMatchFiltersSearch, ImgChampion};
use bitcode::{Decode, Encode};
use common::consts::champion::Champion;
use itertools::Itertools;
use leptos::either::Either;
use leptos::prelude::*;
use leptos::{component, view, IntoView};

#[component]
pub fn SummonerChampionsPage() -> impl IntoView {
    let summoner = expect_context::<Summoner>();
    let sse_match_update_version = expect_context::<ReadSignal<Option<SSEMatchUpdateVersion>>>();
    let meta_store = expect_context::<reactive_stores::Store<MetaStore>>();
    let match_filters_updated = expect_context::<RwSignal<BackEndMatchFiltersSearch>>();
    let (table_sort, set_table_sort) =
        signal::<(TableSortType, bool)>((TableSortType::default(), true));
    let current_sort_type = move || table_sort.get().0;
    let current_sort_normal_flow = move || table_sort.get().1;

    let champions_resource = Resource::new_bitcode(
        move || {
            (
                sse_match_update_version.get().unwrap_or_default(),
                match_filters_updated.get(),
                summoner.id,
            )
        },
        |(_, filters, summoner_id)| async move {
            //println!("{:?} {:?} {:?}", filters, summoner, page_number);
            get_champions(summoner_id, Some(filters)).await
        },
    );

    let toggle_sort = move |sort_type: TableSortType| {
        let (sort, is_desc) = table_sort.get();
        if sort == sort_type {
            set_table_sort((sort_type, !is_desc));
        } else {
            set_table_sort((sort_type, true));
        }
    };

    meta_store.title().set(format!(
        "{}#{} | Champions | Ruche",
        summoner.game_name.as_str(),
        summoner.tag_line.as_str()
    ));
    meta_store.description().set(format!("Discover the top champions played by {}#{} on League Of Legends. Access in-depth statistics, win rates, and performance insights on Ruche, powered by Rust for optimal performance.", summoner.game_name.as_str(), summoner.tag_line.as_str()));
    meta_store
        .url()
        .set(format!("{}?tab=champions", summoner.to_route_path()));
    view! {
        <div>
            <Transition fallback=move || {
                view! { <div class="text-center">Loading Champions</div> }
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
                                                    <col width="160" />
                                                    <col width="140" />
                                                    <col width="105" />
                                                    <col width="72" />
                                                    <col width="66" />
                                                    <col width="66" />
                                                    <col width="66" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                    <col width="48" />
                                                </colgroup>
                                                <thead>
                                                    <tr class="bg-gray-800 text-sm h-[32px]">
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::Index
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("text-center")
                                                            >
                                                                #
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::Champion
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Champion
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::WinRate
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Win Rate
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgKDA
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg KDA
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgGold
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg Gold
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgCs
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from("pl-2 text-left")
                                                            >
                                                                Avg Cs
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgDamageDealt
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Avg Damage Dealt
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::AvgDamageTaken
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Avg Damage Taken
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::DoubleKills
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Double kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::TripleKills
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Triple kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::QuadraKills
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
                                                                class=String::from(
                                                                    "text-ellipsis whitespace-nowrap overflow-hidden",
                                                                )
                                                            >
                                                                Quadra kills
                                                            </TableHeaderItem>
                                                        </th>
                                                        <th class="border border-gray-700 height-inherit">
                                                            <TableHeaderItem
                                                                sort_type=TableSortType::PentaKills
                                                                current_sort_type=move || current_sort_type()
                                                                current_sort_normal_flow=move || current_sort_normal_flow()
                                                                toggle_sort
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
                                                                    let (sort_type, normal_flow) = table_sort.get();
                                                                    sort_type.sort(idx_a, a, idx_b, b, normal_flow)
                                                                })
                                                        }
                                                        key=|(_, champion)| champion.champion_id
                                                        let:champion_with_index
                                                    >
                                                        {
                                                            let (index, champion): (usize, ChampionStats) = champion_with_index;
                                                            let champion_enum = Champion::from(champion.champion_id);
                                                            view! {
                                                                <tr class="p-1">
                                                                    <td class="text-center bg-gray-800 border border-gray-700">
                                                                        {index + 1}
                                                                    </td>
                                                                    <td class="text-left border border-gray-800">
                                                                        <div class="flex items-center">
                                                                            <ImgChampion
                                                                                champion=champion_enum
                                                                                parent_class="my-1 w-8 h-8 sprite-wrapper".to_string()
                                                                                class="rounded-full scale-66 sprite-inner".to_string()
                                                                            />
                                                                            <div class="ml-2 text-center">{champion_enum.to_str()}</div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="text-xs border border-gray-800">
                                                                        {champion.total_wins}W
                                                                        {champion.total_matches - champion.total_wins}L
                                                                        {format_float_to_2digits(champion.win_rate)}%
                                                                    </td>
                                                                    <td class="text-xs border border-gray-800">
                                                                        <div>
                                                                            <div>
                                                                                {calculate_and_format_kda(
                                                                                    champion.avg_kills,
                                                                                    champion.avg_deaths,
                                                                                    champion.avg_assists,
                                                                                )}:1
                                                                            </div>
                                                                            <div>
                                                                                {format!(
                                                                                    "{}/{}/{}",
                                                                                    format_float_to_2digits(champion.avg_kills),
                                                                                    format_float_to_2digits(champion.avg_deaths),
                                                                                    format_float_to_2digits(champion.avg_assists),
                                                                                )}
                                                                            </div>
                                                                        </div>
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {format_with_spaces(champion.avg_gold_earned)}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {champion.avg_cs}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {format_with_spaces(champion.avg_damage_dealt_to_champions)}
                                                                    </td>
                                                                    <td class="border border-gray-800 text-xs">
                                                                        {format_with_spaces(champion.avg_damage_taken)}
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

                                        view! { <div class="text-center">No Champions Found</div> },
                                    ),
                                )
                            }
                        }
                        Err(e) => Err(e),
                    }
                })}
            </Transition>
        </div>
    }
}

#[component]
pub fn TableHeaderItem<S, R, T>(
    sort_type: TableSortType,
    current_sort_type: S,
    current_sort_normal_flow: R,
    toggle_sort: T,
    #[prop(optional)] class: Option<String>,
    children: Children,
) -> impl IntoView
where
    S: Fn() -> TableSortType + Send + Copy + Sync + 'static,
    R: Fn() -> bool + Send + Sync + Copy + 'static,
    T: Fn(TableSortType) + 'static,
{
    view! {
        <button
            class=format!("height-inherit w-full border-blue-500 {} ", class.unwrap_or_default())
            class=(
                "border-t-4",
                move || current_sort_type() == sort_type && !current_sort_normal_flow(),
            )
            class=(
                "border-b-4",
                move || current_sort_type() == sort_type && current_sort_normal_flow(),
            )
            on:click=move |_| toggle_sort(sort_type)
        >
            {children()}
        </button>
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Default)]
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
    pub fn sort(
        &self,
        idx_a: &usize,
        a: &ChampionStats,
        idx_b: &usize,
        b: &ChampionStats,
        is_desc: bool,
    ) -> std::cmp::Ordering {
        // is reversed because we want to sort in descending order
        let ordering = match self {
            TableSortType::Index => idx_b.cmp(idx_a),
            TableSortType::Champion => Champion::from(b.champion_id)
                .to_str()
                .cmp(Champion::from(a.champion_id).to_str()),
            TableSortType::WinRate => (a.win_rate).partial_cmp(&b.win_rate).unwrap(),
            TableSortType::AvgKDA => a.avg_kda.partial_cmp(&b.avg_kda).unwrap(),
            TableSortType::AvgGold => a.avg_gold_earned.partial_cmp(&b.avg_gold_earned).unwrap(),
            TableSortType::AvgCs => a.avg_cs.partial_cmp(&b.avg_cs).unwrap(),
            TableSortType::AvgDamageDealt => a
                .avg_damage_dealt_to_champions
                .partial_cmp(&b.avg_damage_dealt_to_champions)
                .unwrap(),
            TableSortType::AvgDamageTaken => {
                a.avg_damage_taken.partial_cmp(&b.avg_damage_taken).unwrap()
            }
            TableSortType::DoubleKills => a.total_double_kills.cmp(&b.total_double_kills),
            TableSortType::TripleKills => a.total_triple_kills.cmp(&b.total_triple_kills),
            TableSortType::QuadraKills => a.total_quadra_kills.cmp(&b.total_quadra_kills),
            TableSortType::PentaKills => a.total_penta_kills.cmp(&b.total_penta_kills),
        };
        if is_desc {
            ordering.reverse()
        } else {
            ordering
        }
    }
}

#[derive(Clone, Encode, Decode)]
pub struct ChampionStats {
    pub avg_kills: f32,
    pub avg_deaths: f32,
    pub avg_assists: f32,
    pub avg_kda: f32,
    pub win_rate: f32,
    pub avg_gold_earned: u32,
    pub avg_cs: u32,
    pub avg_damage_dealt_to_champions: u32,
    pub avg_damage_taken: u32,
    pub champion_id: u16,
    pub total_matches: u16,
    pub total_wins: u16,
    pub total_double_kills: u16,
    pub total_triple_kills: u16,
    pub total_quadra_kills: u16,
    pub total_penta_kills: u16,
    pub avg_kill_participation: u16,
}
