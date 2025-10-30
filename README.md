<picture>
  <source srcset="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" media="(prefers-color-scheme: dark)">
  <img src="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" alt="Ruche Logo" width="96" height="96">
</picture>

# Ruche — High-Performance League of Legends Stats (Full-Stack Rust)

**Visit: https://ruche.lol**

Ruche is a full-stack Rust web application. It explores a direct question: **Is Rust web-ready, and what does an all-Rust stack look like end-to-end?**  
The result is a production-grade, single-binary site with server-rendered UI, on-demand WASM, native SSE, compact binary payloads, and first-class TLS/H2/H3—**without nginx, Docker, or internal reverse proxies**.

---

## Vision

- **Web-readiness by practice**: demonstrate that **Rust alone** can deliver a complete, modern web product: UI (SSR/CSR), HTTP server, background jobs, storage, and ops.
- **Minimal layers, maximal leverage**: one process, **rustls** TLS, **HTTP/2** and **HTTP/3 (QUIC)**, native compression. Simplicity replaces scaffolding.
- **One language, shared types**: the same Rust types flow across server and client (Leptos + Axum + SQLx), eliminating impedance mismatches.
- **Binary over text**: data moves as **bitcode** compressed with **zstd** (observed to be a small fraction of equivalent JSON sizes in this project), reducing transfer and parse cost.
- **Native real-time**: **SSE** for push updates; background jobs via `tokio::spawn` (Task Director), in-memory caches, and memory-served static assets.
- **Monolith by choice**: no micro-proxies, no container boilerplate required. Raw performance with clear boundaries inside one repository.

---

## Repository Showcase

**Full-Stack Rust**
- **Frontend**: **Leptos** with SSR + CSR hydration, **nested routing**, and **per-route WASM splitting** via `Lazy`.
- **Backend**: **Axum** + **Tokio**, **SQLx** (PostgreSQL), **Riven** (Riot API), SSE broadcaster, sitemap, maintenance tasks.
- **Shared**: `common` crate (strongly-typed LoL enums, asset traits).

**Runtime & Transport**
- **Single binary** serving **TLS (rustls)**, **HTTP/2** by default, **HTTP/3 (QUIC)** in production.
- **Compression**: Brotli and Zstd; no double-compression for SSE/JS/WASM/CSS.
- **Static files**: memory-served after AVIF sprite generation.

**Data & Real-Time**
- **Server functions** return **bitcode + zstd** payloads.
- **SSE** endpoint pushes versioned events for match refresh and live-game state.

**Asset Pipeline**
- Dedicated **asset-generation** CLI fetches DDragon/CommunityDragon, converts to **AVIF**, and builds **CSS sprites** with per-ID classes.

---

## Feature Overview

- Summoner profiles with manual refresh
- Match history with detailed cards and expandable views
- Champion statistics (aggregates and sorting)
- Encounters (with/against breakdown and head-to-head view)
- Live games (team compositions, ranked/champion stats)
- Real-time updates via SSE

---

## Architecture (High Level)

```

Browser (SSR → CSR) ── Leptos (nested routes, Lazy WASM)
│
▼
Axum HTTP
(server functions: bitcode + zstd, static, sitemap, SSE)
│
┌───────────────┴───────────────┐
▼                               ▼
PostgreSQL (SQLx)              Riven (Riot API)
▲                               │
└────────── Task Director (tokio::spawn) ───────► SSE
(match updates, live cache, sitemap, DB maintenance)

```

- **Navigation**: path-based; **no query filters** at the moment.
- **Routes**:
```

/
└── summoners/:platform_route/:summoner_slug
├── (index) → redirects to matches
├── matches
├── champions
├── encounters
├── live
└── encounter/:encounter_platform_route/:encounter_slug

```

---

## Quick Start

See **[docs/INSTALLATION.md](docs/INSTALLATION.md)** for setup, local development, and production build.

---

## Learn More

- Technical highlights (rationale & trade-offs): **[docs/TECHNICAL-HIGHLIGHT.md](docs/TECHNICAL-HIGHLIGHT.md)**
- Asset pipeline (AVIF, sprites, CSS): **[docs/ASSET-PIPELINE.md](docs/ASSET-PIPELINE.md)**
- Full architecture & routing details: **[docs/RUCHE.md](docs/RUCHE.md)**

---

## Contributing

Issues and pull requests are welcome. See `LICENSE`.
