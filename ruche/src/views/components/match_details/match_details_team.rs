use crate::utils::format_with_spaces;
use crate::views::components::match_details::LolMatchParticipantDetails;
use crate::views::ImgChampion;
use common::consts::champion::Champion;

use leptos::prelude::*;
use leptos::{component, view, IntoView};
use std::sync::Arc;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
enum TeamMetric {
    Kills,
    Gold,
    DamageDealt,
    DamageTaken,
    Wards,
    Cs,
}

impl TeamMetric {
    fn all() -> [TeamMetric; 6] {
        [
            TeamMetric::Kills,
            TeamMetric::Gold,
            TeamMetric::DamageDealt,
            TeamMetric::DamageTaken,
            TeamMetric::Wards,
            TeamMetric::Cs,
        ]
    }
    fn label(&self) -> &'static str {
        match self {
            TeamMetric::Kills => "Champion Kills",
            TeamMetric::Gold => "Gold",
            TeamMetric::DamageDealt => "Damage",
            TeamMetric::DamageTaken => "Damage Taken",
            TeamMetric::Wards => "Wards",
            TeamMetric::Cs => "CS",
        }
    }
    fn value(&self, p: &LolMatchParticipantDetails) -> u32 {
        match self {
            TeamMetric::Kills => p.kills as u32,
            TeamMetric::Gold => p.gold_earned,
            TeamMetric::DamageDealt => p.damage_dealt_to_champions,
            TeamMetric::DamageTaken => p.damage_taken,
            TeamMetric::Wards => p.wards_placed as u32,
            TeamMetric::Cs => p.cs as u32,
        }
    }
}

/// Small helper for team-colored classes.
fn team_classes(team_id: u16) -> (&'static str, &'static str) {
    // (bar_bg, header_text)
    if team_id == 100 {
        ("bg-blue-500", "text-blue-400")
    } else {
        ("bg-red-500", "text-red-400")
    }
}

#[component]
pub fn MatchDetailsTeam(match_details: Arc<Vec<LolMatchParticipantDetails>>) -> impl IntoView {
    // Partition into teams + decide winner/loser columns
    let derived = Memo::new({
        let match_details = match_details.clone();
        move |_| {
            let team100: Vec<_> = match_details
                .iter()
                .filter(|p| p.team_id == 100)
                .cloned()
                .collect();
            let team200: Vec<_> = match_details
                .iter()
                .filter(|p| p.team_id == 200)
                .cloned()
                .collect();

            // Determine winners from any participant flag (all teammates share it)
            let team100_won = team100.iter().any(|p| p.won);
            let team200_won = team200.iter().any(|p| p.won);

            // Left = winning team, Right = losing team
            if team100_won && !team200_won {
                ((team100, 100u16, true), (team200, 200u16, false))
            } else if team200_won && !team100_won {
                ((team200, 200u16, true), (team100, 100u16, false))
            } else {
                // Fallback: blue on left if result inconsistent
                (
                    (team100, 100u16, team100_won),
                    (team200, 200u16, team200_won),
                )
            }
        }
    });

    // Selected metric tab
    let (metric, set_metric) = signal(TeamMetric::Kills);

    // Precompute per-metric totals and maxima for both teams
    let totals = Memo::new({
        move |_| {
            let ((left, left_id, _), (right, right_id, _)) = derived();
            let compute = |m: TeamMetric, slice: &Vec<LolMatchParticipantDetails>| -> (u32, u32) {
                let max = slice.iter().map(|p| m.value(p)).max().unwrap_or(0);
                let sum = slice.iter().map(|p| m.value(p)).sum::<u32>();
                (max, sum)
            };
            let mut out = std::collections::HashMap::new();
            for m in TeamMetric::all() {
                out.insert(
                    m,
                    ((compute(m, &left), left_id), (compute(m, &right), right_id)),
                );
            }
            out
        }
    });

    // UI
    view! {
        <div class="my-card w-full">
            // Metric tabs
            <div class="flex flex-wrap gap-2 mb-3">
                {TeamMetric::all()
                    .into_iter()
                    .map(|m| {
                        let is_active = move || metric() == m;
                        view! {
                            <button
                                class=("active-tab", is_active)
                                class=("default-tab", move || !is_active())
                                on:click=move |_| set_metric(m)
                            >
                                {m.label()}
                            </button>
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>

            // Columns grid
            {move || {
                let ((left, left_id, left_won), (right, right_id, right_won)) = derived();
                let metric_now = metric();
                let (((left_max, left_sum), _), ((right_max, right_sum), _)) = totals()
                    .get(&metric_now)
                    .copied()
                    .unwrap();
                let mut left_sorted = left.clone();
                left_sorted.sort_by_key(|p| std::cmp::Reverse(metric_now.value(p)));
                let mut right_sorted = right.clone();
                right_sorted.sort_by_key(|p| std::cmp::Reverse(metric_now.value(p)));

                // Sort participants by metric desc inside each team

                view! {
                    <div class="grid grid-cols-[1fr_220px_1fr] gap-4 items-start">
                        // Left (Winning) team column
                        {team_column(left_sorted, left_id, left_won, metric_now, left_max)}
                        // Center donut comparison
                        {donut_center(metric_now, left_sum, right_sum, left_id, right_id)}
                        // Right (Losing) team column
                        {team_column(right_sorted, right_id, right_won, metric_now, right_max)}
                    </div>
                }
            }}
        </div>
    }
}

// === View helpers ===

fn team_header(team_id: u16, won: bool) -> String {
    let side = if team_id == 100 {
        "Blue Team"
    } else {
        "Red Team"
    };
    let res = if won { "Victory" } else { "Defeat" };
    format!("{res} ({side})")
}

fn pct_width(value: u32, max: u32) -> f32 {
    if max == 0 {
        0.0
    } else {
        (value as f32 / max as f32) * 100.0
    }
}

fn team_column(
    participants: Vec<LolMatchParticipantDetails>,
    team_id: u16,
    won: bool,
    metric: TeamMetric,
    max_in_team: u32,
) -> AnyView {
    let (bar_bg, header_text) = team_classes(team_id);

    view! {
        <div class="space-y-2">
            <div class=format!(
                "text-sm font-semibold {}",
                header_text,
            )>{team_header(team_id, won)}</div>

            <ul class="space-y-1">
                {participants
                    .into_iter()
                    .map(|p| {
                        let champion = Champion::try_from(p.champion_id).unwrap_or_default();
                        let v = metric.value(&p);
                        let w = pct_width(v, max_in_team);
                        let width_style = format!("width:{:.3}%;", w.max(0.0));
                        // keep 0..=100
                        view! {
                            <li>
                                <div class="relative h-8 bg-gray-700 rounded overflow-hidden">
                                    <div
                                        class=format!(
                                            "absolute inset-y-0 left-0 {} rounded",
                                            bar_bg,
                                        )
                                        style=width_style
                                    ></div>

                                    <div class="absolute inset-0 px-2 flex items-center justify-between">
                                        <div class="flex items-center gap-1 min-w-0">
                                            <ImgChampion
                                                champion=champion
                                                parent_class="sprite-wrapper top-[-4px] left-[-4px] w-4 h-4 relative inline-block shrink-0 mr-2"
                                                    .to_string()
                                                class="sprite-inner self-scale-53 rounded-full block"
                                                    .to_string()
                                            />
                                            <span class="truncate">{p.game_name.clone()}</span>
                                        </div>
                                        <span class="ml-2 tabular-nums">
                                            {format_with_spaces(v)}
                                        </span>
                                    </div>
                                </div>
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }.into_any()
}

fn donut_center(
    metric: TeamMetric,
    left_sum: u32,
    right_sum: u32,
    left_id: u16,
    right_id: u16,
) -> AnyView {
    let total = (left_sum as u64 + right_sum as u64) as f64;
    let (lp, rp) = if total == 0.0 {
        (0.0, 0.0)
    } else {
        (left_sum as f64 / total, right_sum as f64 / total)
    };
    let radius = 48.0_f64;
    let stroke = 14.0_f64;
    let c = std::f64::consts::PI * 2.0 * radius;
    let left_len = c * lp;
    let right_len = c * rp;

    let (left_bar, _) = team_classes(left_id);
    let (right_bar, _) = team_classes(right_id);
    let left_color = if left_id == 100 { "#5384E8" } else { "#E84057" };
    let right_color = if right_id == 100 {
        "#5384E8"
    } else {
        "#E84057"
    };

    view! {
        <div class="flex flex-col items-center justify-center text-sm">
            <div class="text-center font-semibold mb-1">{metric.label()}</div>
            <svg width="140" height="140" viewBox="0 0 120 120" class="block">
                // background ring
                <circle
                    cx="60"
                    cy="60"
                    r=radius
                    class="fill-none stroke-gray-800"
                    stroke-width=stroke
                />
                // left arc (starts at 12 o'clock)
                <circle
                    cx="60"
                    cy="60"
                    r=radius
                    class="fill-none"
                    stroke=left_color
                    stroke-width=stroke
                    stroke-linecap="butt"
                    stroke-dasharray=format!("{:.6} {:.6}", left_len, c - left_len)
                    transform="rotate(-90 60 60)"
                />
                // right arc continues after left arc
                <circle
                    cx="60"
                    cy="60"
                    r=radius
                    class="fill-none"
                    stroke=right_color
                    stroke-width=stroke
                    stroke-linecap="butt"
                    stroke-dasharray=format!("{:.6} {:.6}", right_len, c - right_len)
                    transform=format!("rotate({} 60 60)", -90.0 + (lp * 360.0))
                />
                // center label
                <text
                    x="60"
                    y="57"
                    text-anchor="middle"
                    class="fill-gray-200"
                    font-size="12"
                    font-weight="600"
                >
                    {format!("{:.0}%", lp * 100.0)}
                </text>
                <text x="60" y="73" text-anchor="middle" class="fill-gray-400" font-size="10">
                    {format!(
                        "{} / {}",
                        format_with_spaces(left_sum),
                        format_with_spaces(left_sum + right_sum),
                    )}
                </text>
            </svg>
            <div class="mt-1 flex items-center gap-3">
                <span class="flex items-center gap-1">
                    <span class=format!("inline-block w-3 h-3 rounded {}", left_bar)></span>
                    <span>Win</span>
                </span>
                <span class="flex items-center gap-1">
                    <span class=format!("inline-block w-3 h-3 rounded {}", right_bar)></span>
                    <span>Lose</span>
                </span>
            </div>
        </div>
    }
    .into_any()
}
