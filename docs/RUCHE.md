
# Ruche — Full Architecture & Routing

## Contents

- Overview
- App Shell & Meta
- Routing (Nested + Lazy WASM)
- State & SSE Signals
- Data Serialization & Transport
- Backend & Tasks
- Database
- Compression & Transports
- Typical Flows
- Extension Points

## Overview

Crates:

- **common** — LoL enums, asset traits (`HasStaticSrcAsset`, `HasStaticBgAsset`), default sizes, CSS conventions.
- **asset-generation** — fetch DDragon/CommunityDragon, convert to **AVIF**, build **CSS sprites**.
- **ruche** — web app:
    - **Frontend**: Leptos SSR/CSR, router, views, components.
    - **Backend**: Axum HTTP, server functions, SSE broadcaster, Task Director, live cache, sitemap, DB access.
    - **Infra**: rustls TLS; **HTTP/2** default, **HTTP/3 (QUIC)** in production.

## App Shell & Meta

- `shell(LeptosOptions)` emits `<!DOCTYPE html>` with `<AutoReload/>`, `<HydrationScripts/>`, `<MetaTags/>`.
- `MetaStore` in context powers `<Title/>`, `description`, OpenGraph tags, canonical `<link>` using `SITE_URL`.
- Base stylesheet: `<Stylesheet id="leptos" href="/pkg/ruche.css" />`.

## Routing (Nested + Lazy WASM)

Path-based navigation; **no query filters**.

```

/
└── summoners/:platform_route/:summoner_slug          (ParentRoute: SummonerPageRoute)
├── (index) → redirects to matches
├── matches                                       (Lazy::<SummonerMatchesRoute>)
├── champions                                     (Lazy::<SummonerChampionsRoute>)
├── encounters                                    (Lazy::<SummonerEncountersRoute>)
├── live                                          (Lazy::<SummonerLiveRoute>)
└── encounter/:encounter_platform_route/:encounter_slug
(Lazy::<SummonerEncounterRoute>)

```

- Parent container provides shared context (SSE signals, meta, optional filters).
- Each child route is **lazy-loaded** as a separate WASM chunk and hydrates when active.

## State & SSE Signals

- `SSEMatchUpdateVersion: RwSignal<Option<_>>`
- `SSEInLiveGame: RwSignal<_>`
- Optional filter store: `Store<BackEndMatchFiltersSearch>`; internal only, not URL-synced.

## Data Serialization & Transport

- Server functions (`#[server]`) share Rust types across SSR/CSR.
- Responses: **bitcode + zstd**; client decodes compact binary payloads.

## Backend & Tasks

- **Axum** serves static assets and server-function endpoints; sitemap endpoints (`/sitemap-index.xml`).
- **SSE**: `/sse/match_updated/{platform}/{summoner_id}`
  - `1:{n}` → `SummonerMatches(n)` (clients refresh)
  - `0:{v}` → `LiveGame(Some(v))`
  - `0:`     → `LiveGame(None)`
- **Task Director** (`tokio::spawn`):
  - Update matches & timelines; mark trashed; emit SSE
  - Sync pro players
  - Refresh live cache; purge expired entries
  - Cleanup inactive SSE senders
  - Generate sitemap(s)
  - Daily DB maintenance

## Database

- PostgreSQL via SQLx; representative tables: `summoners`, `lol_matches`, `lol_match_participants`, timeline tables.
- Indexes on common access paths; bulk ops via `UNNEST`; compile-time `DB_CHUNK_SIZE` governs batch sizes.

## Compression & Transports

- Brotli/Zstd; **no double-compression** for SSE/JS/WASM/CSS.
- TLS via rustls; **HTTP/2** and **HTTP/3 (QUIC)** with ALPN/`Alt-Svc`.

## Typical Flows

**Summoner + Matches**
1. Navigate to `/summoners/:platform_route/:summoner_slug` (index → `matches`).
2. Child route loads lazily and renders data from server functions (bitcode/zstd).
3. Background updates complete; SSE version bump triggers client refresh.

**Live Game**
1. Open `live` tab; server function resolves via Riot API (Riven), enriches with ranked/champion stats, updates cache.
2. Client renders; periodic refresh + SSE maintain freshness.

## Extension Points

- Add nested routes under the summoner parent; use `Lazy::<...>::new()` to preserve WASM splitting.
- Register new background jobs in the Task Director.
- Extend asset groups and CSS conventions via `common` and the generator.