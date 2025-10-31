use std::time::Duration;
use axum::extract::{Path, State};
use axum::response::Sse;
use axum::response::sse::{Event, KeepAlive};
use dashmap::DashMap;
use tokio::sync::broadcast::Sender;
use tokio::time;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use common::consts::platform_route::PlatformRoute;
use crate::backend::server_fns::get_encounter::ssr::find_summoner_puuid_by_id;
use crate::backend::server_fns::get_live_game::ssr;
use crate::ssr::AppState;
use crate::utils::{Puuid, SSEEvent};


pub type SubscriberMap = DashMap<i32, Sender<SSEEvent>>;
pub async fn sse_broadcast_match_updated(
    Path((platform_route, summoner_id)): Path<(String, i32)>,
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let (rx, mut pending_event) = {
        let mut in_live_game = state
            .live_game_cache
            .summoner_id_to_game
            .get(&summoner_id)
            .is_some();
        // check if existing channel exists
        let entry = state
            .summoner_updated_sender
            .entry(summoner_id)
            .or_insert_with(|| {
                let (sender, _) = tokio::sync::broadcast::channel(3);
                if state
                    .live_game_cache
                    .summoner_id_to_game
                    .get(&summoner_id)
                    .is_some()
                {
                    in_live_game = true;
                } else {
                    // fetch first time live game data
                    let inner_sender = sender.clone();
                    let db = state.db.clone();
                    let riot_api = state.riot_api.clone();
                    let live_game_cache = state.live_game_cache.clone();
                    let platform_route = PlatformRoute::from_code(platform_route.as_str()).unwrap();
                    tokio::spawn(async move {
                        let puuid = Puuid::new(
                            find_summoner_puuid_by_id(&db, summoner_id)
                                .await
                                .unwrap()
                                .as_str(),
                        );
                        let live_game =
                            ssr::get_live_game_data(&db, &riot_api, puuid, platform_route)
                                .await
                                .unwrap();
                        if let Some((summoner_ids, live_game)) = live_game {
                            live_game_cache.set_game_data(
                                live_game.game_id,
                                summoner_ids,
                                live_game,
                            );
                            inner_sender.send(SSEEvent::LiveGame(Some(1))).unwrap();
                        } else {
                            inner_sender.send(SSEEvent::LiveGame(None)).unwrap();
                        }
                    });
                }

                sender
            });

        (
            entry.value().subscribe(),
            in_live_game.then_some( SSEEvent::LiveGame(Some(1))),
        )
    };
    let mut summoner_matches_update_count = 0u16;
    let mut summoner_live_game_version_update_count = 0u16;
    let debounce_interval = Duration::from_millis(500);

    let stream = async_stream::stream! {

                  let mut event_id: u64 = 0;
               // Use an interval timer to enforce the 1-second delay
               let mut interval = time::interval(debounce_interval);
               interval.set_missed_tick_behavior(time::MissedTickBehavior::Delay);
               interval.reset();


               // Wrap the receiver in a stream
               let mut rx_stream = BroadcastStream::new(rx);

               loop {
                   tokio::select! {
                       message = rx_stream.next() =>{
                           pending_event= match message{
                               Some(Ok(SSEEvent::SummonerMatches(_))) => {
                                    summoner_matches_update_count += 1;
                                   Some(SSEEvent::SummonerMatches(summoner_matches_update_count))
                               }
                               Some(Ok(SSEEvent::LiveGame(Some(_))) ) => {
                                   summoner_live_game_version_update_count += 1;
                                   Some(SSEEvent::LiveGame(Some(summoner_live_game_version_update_count)))
                               }
                               Some(Ok(SSEEvent::LiveGame(None))) => {
                                   Some(SSEEvent::LiveGame(None))
                               }
                               _ => None,
                           };
                       }
                       _ = interval.tick() => {
                             if let Some(event) = pending_event.take() {
                                   event_id = event_id.wrapping_add(1);
                                    yield Ok(
                                        Event::default()
                                            .id(event_id.to_string())
                                            .data(event.to_string())
                                            .retry(Duration::from_millis(3000))
                                    );
                             }
                       }
                       else => {
                           // Stream has ended
                           break;
                       }
                   }
               }
           };

    Sse::new(stream).keep_alive(KeepAlive::default())
}