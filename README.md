<picture>
  <source srcset="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" media="(prefers-color-scheme: dark)">
  <img src="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" alt="Ruche Logo">
</picture>

# Ruche — High-Performance League of Legends Stats
**Visit us at [ruche.lol](https://ruche.lol)**

Ruche is a fast, scalable League of Legends statistics platform inspired by industry leaders (e.g., OP.GG). It provides summoner profiles, match histories, champion stats, live games, and encounter analytics—with a focus on low latency, efficient data fetching, and real-time updates.

---

## Key Features

### Summoner Profiles
- **Search** by **Game Name**, **Tag Line**, and **Platform**. If a summoner is missing, Ruche fetches from Riot’s API, stores it, and redirects to the profile.
- **Profile overview** with name, level, icon, and pro status.  
  Manual **Update** button to refresh data and trigger background match updates.
- **Tabs**
  - **Matches**: detailed cards, participants, and inline details.
  - **Champions**: aggregated per-champion stats with sorting.
  - **Encounters**: who you play **with**/**against**; search, stats, and history.
  - **Live**: show current game (if any) with a green tab indicator.
  - **Encounter**: in-depth head-to-head page for a chosen player.

### Match History
- **Cards** summarizing result, K/D/A, KP, items, and participants.
- **Badges**
  - **Encounter count** (green) for frequent teammates/opponents.
  - **Pro** (purple) linking to LolPros.gg when detected.
- **Expandable details**
  - **Overview**: team & player metrics.
  - **Team**: reserved for deeper team insights (roadmap).
  - **Build**: timeline of item buys/sells, skills order, and perks.

### Champion Statistics
- Totals, win rate, averages (K/D/A, gold, CS, damage dealt/taken), multi-kills.
- Sort by **Win Rate**, **Avg KDA**, **Gold**, **CS**, **Damage Dealt**, **Damage Taken**, **Multi-kills**.

### Encounters
- **List** the summoners you’ve played with/against most.
- **Search** within your encounter list.
- **Details**: With/Against tabs, side-by-side stats, and the full shared match list with filters.

### Live Games
- Real-time game info: queue, map, elapsed time, participants.
- **Live indicator**: the “Live” tab turns green if the summoner is in game.
- **Auto cache refresh** at `LIVE_GAME_CACHE_UPDATE_INTERVAL` (configurable); **Refresh** button bypasses cache.
- **Per participant**: champion, spells, runes, encounter counts, ranked stats, champion stats.

---

## Real-Time Updates (SSE)

- **Endpoint**: `/sse/match_updated/{platform}/{summoner_id}`
- **Events** (compact payloads):
  - `1:{n}` → `SummonerMatches(n)`: version increments; UI reloads matches/champions/encounters.
  - `0:{v}` → `LiveGame(Some(v))`
  - `0:`    → `LiveGame(None)`
- **Debounce** ~**500 ms**; inactive subscriptions are cleaned periodically.

---

## Global Filtering & URLs

Shared filters are persisted in the URL:

- **Query keys**
  - `filters[champion_id]`
  - `filters[queue_id]`
  - `filters[start_date]` (`YYYY-MM-DD`)
  - `filters[end_date]` (`YYYY-MM-DD`)
- **Routes**
  - Summoner: `/platform/{PLATFORM}/summoners/{GAME_NAME}-{TAG_LINE}`
  - Tabs: `?tab=matches|champions|encounters|live|encounter`
  - Encounter detail:  
    `?tab=encounter&encounter_slug={GAME}-{TAG}&encounter_platform={PLATFORM}`
  - Pagination: `?page={u16}`

---

## Technical Highlights

### Frontend
- **Leptos** (Rust) with SSR/CSR hybrid, reactive signals, and context providers.
- **Components**: `SummonerSearchPage`, `SummonerNav`, `MatchFilters`, `Pagination`, `MatchDetails` (Overview/Build/Team), and pages for Matches/Champions/Encounters/Encounter/Live.
- **Serialization**: **Bitcode** payloads (small and fast) with zstd compression.
- **UX**: accessible controls, fast navigation, URL-synced state.

### Backend
- **Axum + Tokio** (async), **SQLx + PostgreSQL**, **Riven** (Riot API).
- **Task Director**: a priority scheduler for:
  - **Update Matches**
  - **Update Pro Players**
  - **Sitemap generation**
  - **SSE broadcaster cleanup**
  - **Live game cache management**
  - **DB maintenance**
- **Live game cache**: in-memory (DashMap) with TTL and on-read elapsed recompute.
- **Compression**: Brotli + Zstd (no double-compress for SSE/JS/WASM/CSS).
- **TLS & transports**
  - **Rustls** for HTTPS (**HTTP/2**)
  - **HTTP/3 (QUIC)** enabled in production with automatic `Alt-Svc`
  - HTTP→HTTPS redirection on port 80

### Common crate (shared types)
- Strongly-typed LoL enums & helpers in `common`:
  - `champion::Champion` (+ `CHAMPION_OPTIONS`)
  - `queue::Queue` (+ `QUEUE_OPTIONS`) and display names
  - `map::Map`, `game_mode::GameMode`, `perk::Perk`, `summoner_spell::SummonerSpell`
  - `platform_route::PlatformRoute` (+ conversions)
  - `item::Item`, `profile_icon::ProfileIcon`
- **Traits** for asset wiring:
  - `HasStaticSrcAsset` → builds `/assets/{path}/{id}.avif` URLs (e.g., profile icons).
  - `HasStaticBgAsset`  → builds CSS class names for sprites (e.g., items/spells/perks/champions).
- `AssetType` (Item | ProfileIcon | SummonerSpell | Perk | Champion) provides:
  - Path segments (`items`, `profile_icons`, `summoner_spells`, `perks`, `champions`)
  - Default CSS class prefixes (`ii`, `pi`, `ss`, `pk`, `cn`)
  - Default sizes (px): Items **22×22**, Spells **22×22**, Perks **28×28**, Champions **48×48**, Profile Icons **64×64**
- All the above derive **bitcode** `Encode/Decode` for fast SSR<→CSR transfer.

---

## Asset Pipeline (Sprites, AVIF, CSS)

Ruche ships a **custom asset-generation** binary that downloads static assets (from DDragon/CommunityDragon), converts to **AVIF**, and builds **CSS sprites** for efficient delivery.

### Sources
- **Versions**: `https://ddragon.leagueoflegends.com/api/versions.json` (latest = index 0)
- **Items**: `.../cdn/{version}/data/en_US/item.json` + `.../img/item/{id}.png`
- **Spells**: `.../cdn/{version}/data/en_US/summoner.json` + `.../img/spell/{name}.png`
- **Perks**: CommunityDragon perks JSON + `.../cdn/{version}/data/en_US/runesReforged.json`
- **Champions (square)**: `https://cdn.communitydragon.org/{version}/champion/{id}/square`
- **Profile icons**: `.../cdn/{version}/data/en_US/profileicon.json` + `.../img/profileicon/{id}.png`

### Paths
- **Temp** (PNG): `asset-generation/tmp/{items|summoner_spells|perks|champions|profile_icons}`
- **Final assets (AVIF)**: `ruche/public/assets/...`
- **Stylesheets**: `ruche/style/{items|summoner_spells|perks|champions}.css`
- **Sprite files (AVIF)**: `/assets/{items|summoner_spells|perks|champions}.avif`
- **Logo**: AVIF written to `/assets/logo.avif` from `asset-generation/tmp/logo.png`

### What gets sprited vs standalone?
- **Sprited** (**background-image**): **Items**, **Summoner Spells**, **Perks**, **Champions**
- **Standalone images** (**<img src>**): **Profile Icons**, **Logo**

### Image processing
- **Resize** (Lanczos3) to default sizes (see *Common crate* above)
- **AVIF encoder**: quality **75**, speed **1** (logo uses quality **100**)
- Max concurrency for downloads (reqwest): **10**, with **exponential backoff** retries (3 attempts)

### Sprite layout
- Square grid `ceil(sqrt(n)) × ceil(sqrt(n))`
- One **.avif** sprite per asset type
- Generated CSS includes **class per ID** with `background-position`, `width`, `height`

### CSS class naming
- Prefix = asset type; suffix = numeric **ID**
  - Items: `.ii-<itemId>`
  - Spells: `.ss-<spellId>`
  - Perks: `.pk-<perkId>`
  - Champions: `.cn-<championId>`
  - (Profile icons use `<img src="/assets/profile_icons/<id>.avif">`)

#### Usage examples
```html
<!-- Items (e.g., Infinity Edge 3031) -->
<span class="ii-3031" aria-label="Infinity Edge"></span>

<!-- Spells (Flash 4) -->
<span class="ss-4" aria-label="Flash"></span>

<!-- Perks (First Strike 8369) -->
<i class="pk-8369" aria-label="First Strike"></i>

<!-- Champions (Aatrox 266) -->
<div class="cn-266" title="Aatrox"></div>

<!-- Profile icon -->
<img src="/assets/profile_icons/1234.avif" alt="Profile Icon #1234" width="64" height="64">
````

### CLI (asset-generation)

```bash
# Generate everything (download missing, convert & build sprites as needed)
cargo run --bin asset-generation --release

# Force-rebuild specific groups
cargo run --bin asset-generation --release -- \
  --items \
  --summoner-spells \
  --perks \
  --champions \
  --profile-icons \
  --logo

# See flags
cargo run --bin asset-generation --release -- --help
```

> The generator only converts/builds when something **actually changed**. You can force a rebuild per group with flags above.

---

## Security & SEO

* **TLS (Rustls)** with modern ciphers; automatic HTTP→HTTPS redirect.
* **HTTP/3 (QUIC)** for lower latency; H2/H1.1 fallback.
* **SEO**

    * Per-page meta (title/description/image)
    * Clean, canonical URLs
    * **Sitemap**: generated daily at `/sitemap-index.xml`

---

## Database

* **Migrations** run automatically on startup when using split DB vars (see **Configuration**).
* **Indexes** (examples): `summoners(puuid)`, `summoners(game_name, lower(tag_line), platform)`,
  `lol_matches(match_id)`, `lol_matches(match_end, queue_id)`,
  `lol_match_participants(summoner_id, lol_match_id)`
* **Maintenance**: regular `VACUUM ANALYZE` (avoid `VACUUM FULL` in routine ops).
* **Batching**: `DB_CHUNK_SIZE` is a **compile-time constant** (default **500**).

> `DB_CHUNK_SIZE` is **not** configurable via `.env` and remains a const.

---

## Installation

If you encounter WASM issues, ensure `wasm-*` versions are consistent between the repo `Cargo.toml`, `cargo-leptos`, and the Leptos crate.

### Requirements

* **Rust** (via [rustup](https://rustup.rs/)), **nightly** toolchain
* **Node.js 18+** (assets & Tailwind)
* **NASM** (for `ravif` AVIF encoding)
* **PostgreSQL**
* **Riot API Key** from the [Riot Developer Portal](https://developer.riotgames.com/)

### Setup

```bash
# Install Rust nightly and the WASM target
rustup default nightly
rustup target add wasm32-unknown-unknown

# Install wasm-opt (recommended if you hit "failed to grow table")
cargo install wasm-opt

# Install cargo-leptos
cargo install cargo-leptos

# Clone the repository
git clone https://github.com/gqsnt/ruche.git
cd ruche

# Copy and configure environment
cp .env.example .env

# Install tailwindcss CLI
cd ruche
npm install tailwindcss @tailwindcss/cli
cd ..

# Generate assets and CSS sprites (see --help for flags)
cargo run --bin asset-generation --release
```

### Local Development

```bash
# Ensure ENV=DEV in .env
# Run Leptos (SSR dev server + hot reload)
cargo leptos watch
```

---

## Production Build & Deployment (HTTP/2 + HTTP/3)

### Requirements

* **Let’s Encrypt** / **Certbot** for TLS certificates.

### Network & Firewall

* Open **TCP:80** (ACME) and **TCP:443** (HTTPS).
* Open **UDP:443** for **HTTP/3 (QUIC)**.
* HTTP redirector listens on :80 and forwards to HTTPS.

### Setup

> The repository is expected under `/etc/ruche` for default service paths.

```bash
# Stop and disable nginx if present (Ruche serves HTTPS/H3 itself)
sudo systemctl stop nginx
sudo systemctl disable nginx
killall nginx || true

# Firewall
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw allow 443/udp  # required for HTTP/3 (QUIC)

# Issue a certificate and set LETS_ENCRYPT_PATH in .env to its directory
sudo certbot certonly --standalone -d your_domain.com

# Install systemd service
sudo cp /etc/ruche/ruche.service /etc/systemd/system/ruche.service
sudo systemctl enable ruche

# Pull, (re)build assets if needed, rebuild the project, swap binaries, restart
sh rebuild.sh
```

---

## PostgreSQL Setup

```bash
# Create the database
sudo -u postgres psql -c "CREATE DATABASE ruche;"

# Set the password (align with your .env)
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'password';"

# Allow socket connections in pg_hba.conf (adjust path to your distro)
sudo -u postgres psql -c "SHOW hba_file;"
# edit: choose your local auth method (e.g., trust/scram)
sudo nano /etc/postgresql/17/main/pg_hba.conf

# Restart
sudo systemctl restart postgresql
```

---

## Configuration

> Choose **one** DB configuration method: a single `DATABASE_URL`, **or** split variables (split runs migrations at startup).

| Variable                                                 | Type           | Default | Scope   | Description                                                 |
| -------------------------------------------------------- | -------------- | ------: | ------- | ----------------------------------------------------------- |
| `ENV`                                                    | `DEV` | `PROD` |   `DEV` | global  | Execution mode (ports, tasks).                              |
| `RIOT_API_KEY`                                           | string         |       – | backend | Riot API key used by Riven.                                 |
| `DATABASE_URL`                                           | URL            |       – | backend | PostgreSQL connection string (recommended in prod).         |
| `DB_USER_NAME` / `DB_PASSWORD` / `DB_NAME` / `DB_SOCKET` | strings        |       – | backend | Alternative to `DATABASE_URL` (runs migrations on startup). |
| `MAX_PG_CONNECTIONS`                                     | int            |    `10` | backend | Pool size when using split DB vars.                         |
| `MAX_MATCHES`                                            | int            |  `1500` | backend | Soft limit when fetching historical match IDs.              |
| `MATCH_TASK_UPDATE_INTERVAL`                             | seconds        |     `5` | backend | Background match update interval.                           |
| `LIVE_GAME_CACHE_UPDATE_INTERVAL`                        | seconds        |    `30` | backend | Live-game cache refresh interval.                           |
| `LOL_PRO_TASK_ON_STARTUP`                                | bool           | `false` | backend | Sync pro players on startup.                                |
| `SITE_MAP_TASK_ON_STARTUP`                               | bool           | `false` | backend | Generate sitemap on startup.                                |
| `LETS_ENCRYPT_PATH`                                      | path           |       – | prod    | Folder with `fullchain.pem` & `privkey.pem`.                |

> `DB_CHUNK_SIZE` remains a **const** at compile time (default **500**).

---

## Contributing

1. **Fork** the repository
2. **Create a branch** for your feature/bugfix
3. Write **clear, concise commits**
4. Open a **Pull Request** describing your change

---

## Acknowledgements

* **Riot Games** for APIs and assets
* **Leptos** for the reactive Rust SSR/CSR framework
* The **open source** ecosystem

---

## Future Roadmap

* **Team tab** in match details (deeper composition & team stats)
* **Mobile optimization** (responsiveness & touch ergonomics)
* **Branding** improvements (visual identity)


