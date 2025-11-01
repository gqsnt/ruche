use std::convert::Infallible;
use std::sync::{Arc, Weak};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive};
use axum::response::Sse;
use dashmap::{DashMap, DashSet};
use leptos::logging::log;
use tokio::sync::watch;

use crate::backend::server_fns::get_encounter::ssr::find_summoner_puuid_by_id;
use crate::backend::server_fns::get_live_game::ssr;
use crate::ssr::AppState;
use crate::utils::Puuid;
use common::consts::platform_route::PlatformRoute;

const GC_GRACE_MS: u64 = 10_000;
const DEBOUNCE_MS: u64  = 500;
#[inline]
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}


#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub struct SseSnapshot {
    pub live_ver:  u64, // 0 = pas de partie ; >0 = epoch live
    pub match_ver: u64, // monotone
}

impl SseSnapshot{
    pub fn new(
        live_ver:u64,
        match_ver:u64
    ) -> Self{
        Self{
            live_ver,
            match_ver,
        }
    }
}


#[derive(Clone)]
pub struct Topic {
    tx: watch::Sender<SseSnapshot>,
    state: Arc<KeyState>,
    subs: Arc<AtomicUsize>,
    last_empty_at: Arc<AtomicU64>,
    last_sent_at: Arc<AtomicU64>,
    pending:     Arc<AtomicBool>,
}

#[derive(Default)]
pub struct KeyState {
    live_ver:  std::sync::atomic::AtomicU64,
    match_ver: std::sync::atomic::AtomicU64,
}

pub struct Hub {
    pub topics: DashMap<i32, Topic>,
    pub dirty:  DashSet<i32>,
    gc: DashSet<i32>,
}

impl Hub {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { topics: DashMap::new(), dirty: DashSet::new(), gc: DashSet::new() })
    }

    fn or_insert_topic(&self, sid: i32) -> Topic {
        self.topics.entry(sid).or_insert_with(|| {
            let (tx, _rx) = watch::channel(SseSnapshot::default());
            Topic {
                tx,
                state: Arc::new(KeyState::default()),
                subs: Arc::new(AtomicUsize::new(0)),
                last_empty_at: Arc::new(AtomicU64::new(0)),
                last_sent_at: Arc::new(AtomicU64::new(0)),
                pending: Arc::new(AtomicBool::new(false)),
            }
        }).clone()
    }

    #[inline]
    fn mark_pending(&self, sid: i32) {
        if let Some(t) = self.topics.get(&sid) {
            t.pending.store(true, Ordering::SeqCst);
            self.dirty.insert(sid);
        }
    }



    // Garde la version simple si vous en avez besoin ailleurs
    pub fn subscribe(&self, sid: i32) -> watch::Receiver<SseSnapshot> {
        self.or_insert_topic(sid).tx.subscribe()
    }

    pub fn bump_matches(&self, sid: i32) {
        if let Some(t) = self.topics.get(&sid) {
            t.state.match_ver.fetch_add(1, Ordering::SeqCst);
        }
        self.mark_pending(sid);
    }

    pub fn ensure_live_present(&self, sid: i32) {
        if let Some(t) = self.topics.get(&sid) {
            if t.state.live_ver.load(Ordering::SeqCst) == 0 {
                t.state.live_ver.store(1, Ordering::SeqCst);
            }
        }
        self.mark_pending(sid);
    }

    pub fn bump_live_epoch(&self, sid: i32) {
        if let Some(t) = self.topics.get(&sid) {
            let cur = t.state.live_ver.load(Ordering::SeqCst);
            let next = if cur == 0 { 1 } else { cur.saturating_add(1) };
            t.state.live_ver.store(next, Ordering::SeqCst);
        }
        self.mark_pending(sid);
    }

    pub fn set_live_none(&self, sid: i32) {
        if let Some(t) = self.topics.get(&sid) {
            t.state.live_ver.store(0, Ordering::SeqCst);
        }
        self.mark_pending(sid);
    }

    /// Tick périodique : publie les snapshots et ramasse les topics sans abonnés
    pub async fn run(self: Arc<Self>, period: Duration) {
        let mut tick = tokio::time::interval(period);
        loop {
            tick.tick().await;
            let now = now_millis();

            // 1) Throttle 500 ms sur les sids "sales"
            let dirty_keys: Vec<i32> = self.dirty.iter().map(|k| *k).collect();
            for sid in dirty_keys {
                if let Some(t) = self.topics.get(&sid) {
                    // si plus d'abonnés: programmer GC et retirer de dirty
                    if t.subs.load(Ordering::SeqCst) == 0 {
                        if t.last_empty_at.load(Ordering::SeqCst) == 0 {
                            t.last_empty_at.store(now, Ordering::SeqCst);
                            self.gc.insert(sid);
                        }
                        self.dirty.remove(&sid);
                        continue;
                    }

                    // si pas de maj en attente: nettoyer dirty
                    if !t.pending.load(Ordering::SeqCst) {
                        self.dirty.remove(&sid);
                        continue;
                    }

                    let last = t.last_sent_at.load(Ordering::SeqCst);
                    let due  = last == 0 || now.saturating_sub(last) >= DEBOUNCE_MS;

                    if due {
                        let snap = SseSnapshot::new(
                            t.state.live_ver.load(Ordering::SeqCst),
                            t.state.match_ver.load(Ordering::SeqCst),
                        );
                        let prev = *t.tx.borrow();
                        if prev != snap {
                            let _ = t.tx.send_replace(snap);
                        }
                        t.last_sent_at.store(now, Ordering::SeqCst);
                        // consommation de la rafale
                        t.pending.store(false, Ordering::SeqCst);
                        // on retire de dirty; une nouvelle rafale le réinsèrera
                        self.dirty.remove(&sid);
                    } else {
                        // pas encore dû: rester dans dirty, on réessaie au prochain tick
                    }
                } else {
                    // topic déjà supprimé
                    self.dirty.remove(&sid);
                }
            }

            // 2) GC avec grâce (via guard RAII)
            let gc_candidates: Vec<i32> = self.gc.iter().map(|k| *k).collect();
            for sid in gc_candidates {
                if let Some(t) = self.topics.get(&sid) {
                    if t.subs.load(Ordering::SeqCst) == 0 {
                        let ts = t.last_empty_at.load(Ordering::SeqCst);
                        if ts != 0 && now.saturating_sub(ts) >= GC_GRACE_MS {
                            // libérer la réf avant remove
                            drop(t);
                            let _ = self.topics.remove(&sid);
                            self.gc.remove(&sid);
                            self.dirty.remove(&sid);
                            log!("Hub GC: removed topic sid={}", sid);
                        }
                    } else {
                        self.gc.remove(&sid);
                    }
                } else {
                    self.gc.remove(&sid);
                    self.dirty.remove(&sid);
                }
            }
        }
    }
}

pub fn subscribe_with_guard(hub: Arc<Hub>, sid: i32) -> (watch::Receiver<SseSnapshot>, SubscriptionGuard) {
    let t = hub.or_insert_topic(sid);
    t.subs.fetch_add(1, Ordering::SeqCst);
    let rx = t.tx.subscribe();
    let guard = SubscriptionGuard {
        hub: Arc::downgrade(&hub),
        sid,
        subs: t.subs.clone(),
        last_empty_at: t.last_empty_at.clone(),
    };
    (rx, guard)
}


pub struct SubscriptionGuard {
    hub: Weak<Hub>,
    sid: i32,
    subs: Arc<AtomicUsize>,
    last_empty_at: Arc<AtomicU64>,
}

impl Drop for SubscriptionGuard {
    fn drop(&mut self) {
        let prev = self.subs.fetch_sub(1, Ordering::SeqCst);
        if prev == 1 {
            self.last_empty_at.store(now_millis(), Ordering::SeqCst);
            if let Some(h) = self.hub.upgrade() {
                h.gc.insert(self.sid);
            }
        }
    }
}



fn snapshot_to_payload(s: &SseSnapshot) -> String {
    // Votre format actuel: "live_ver?:match_ver" (live vide si 0)
    if s.live_ver == 0 {
        format!(":{}", s.match_ver)
    } else {
        format!("{}:{}", s.live_ver, s.match_ver)
    }
}
pub async fn sse_broadcast_match_updated(
    Path((platform_route, summoner_id)): Path<(String, i32)>,
    State(state): State<AppState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let platform = PlatformRoute::from_code(&platform_route).expect("invalid platform");

    // IMPORTANT: on récupère un guard pour décrémenter à la fermeture de la connexion SSE
    let (mut rx, guard) = subscribe_with_guard(state.hub.clone(), summoner_id);

    if state.live_game_cache.get_game_data(summoner_id).await.is_some() {
        state.hub.ensure_live_present(summoner_id);
    } else {
        let hub = state.hub.clone();
        let db = state.db.clone();
        let riot_api = state.riot_api.clone();
        tokio::spawn(async move {
            let puuid = Puuid::new(
                find_summoner_puuid_by_id(&db, summoner_id).await.expect("puuid").as_str(),
            );
            match ssr::get_live_game_data(&db, &riot_api, puuid, platform).await {
                Ok(Some((_ids, _live))) => hub.bump_live_epoch(summoner_id),
                Ok(None) | Err(_) => hub.set_live_none(summoner_id),
            }
        });
    }

    let stream = async_stream::stream! {
        let mut eid: u64 = 0;
        let _keep_guard_alive = guard;
        // Optionnel: envoi immédiat si déjà live
        let first = *rx.borrow();
        if first.live_ver != 0 {
            yield Ok(Event::default()
                .id({eid+=1; eid}.to_string())
                .data(snapshot_to_payload(&SseSnapshot::new(1, 0)))
                .retry(Duration::from_millis(3000)));
        }

        loop {
            if rx.changed().await.is_err() { break; }
            let snap = *rx.borrow();
            yield Ok(Event::default()
                .id({ eid = eid.wrapping_add(1); eid }.to_string())
                .data(snapshot_to_payload(&snap))
                .retry(Duration::from_millis(3000)));
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
