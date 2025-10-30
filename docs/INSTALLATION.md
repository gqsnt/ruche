
# Installation & Setup

Local development and production deployment.

## Requirements

- **Rust (nightly)** + target `wasm32-unknown-unknown`
- **Node.js ≥ 18**
- **NASM** (for `ravif`)
- **PostgreSQL**
- **Riot API key** (Riot Developer Portal)
- Tools: `cargo-leptos` (SSR/CSR builds), `wasm-opt` (recommended)

## Toolchain

```bash
rustup toolchain install nightly
rustup default nightly
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos
# helpful for large WASM:
cargo install wasm-opt
````

## Clone & Configure

```bash
git clone https://github.com/gqsnt/ruche.git
cd ruche
cp .env.example .env
# Edit .env: set RIOT_API_KEY and DB configuration
```

## Frontend dependencies

```bash
cd ruche
npm install
cd ..
```

## Generate Assets

```bash
cargo run --bin asset-generation --release
# flags
cargo run --bin asset-generation --release -- --help
```

## Local Development

```bash
# Ensure ENV=DEV in .env
cargo leptos watch
```

## Production Build

```bash
cargo leptos build --release
```

### TLS & Network

* **rustls** TLS; **HTTP/2** default; **HTTP/3 (QUIC)** in production.
* Open **TCP:80**, **TCP:443**, **UDP:443** (QUIC).
* If using a cert directory, set `LETS_ENCRYPT_PATH` with `fullchain.pem` and `privkey.pem`.

Example (systemd paths are illustrative):

```bash
sudo cp /etc/ruche/ruche.service /etc/systemd/system/ruche.service
sudo systemctl enable ruche
sh rebuild.sh
```

## PostgreSQL

```bash
createdb ruche
# or
sudo -u postgres psql -c "CREATE DATABASE ruche;"
# optional auth setup
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'your_password';"
sudo -u postgres psql -c "SHOW hba_file;"
sudo systemctl restart postgresql
```

**Migrations**
With split DB variables, migrations can run at startup. In production, prefer a single `DATABASE_URL`.

## Configuration

| Variable                          | Type          | Default | Scope   | Description                          |
| --------------------------------- | ------------- | ------: | ------- | ------------------------------------ |
| `ENV`                             | `DEV \| PROD` |   `DEV` | global  | Execution mode.                      |
| `RIOT_API_KEY`                    | string        |       – | backend | Riot API key (Riven).                |
| `DATABASE_URL`                    | URL           |       – | backend | PostgreSQL DSN (preferred in prod).  |
| `DB_USER_NAME` / `DB_PASSWORD`    | strings       |       – | backend | Split DB credentials.                |
| `DB_NAME` / `DB_SOCKET`           | strings       |       – | backend | Split DB database and socket/host.   |
| `MAX_PG_CONNECTIONS`              | int           |      10 | backend | Pool size for split DB mode.         |
| `MAX_MATCHES`                     | int           |    1500 | backend | Soft cap for historical fetch.       |
| `MATCH_TASK_UPDATE_INTERVAL`      | seconds       |       5 | backend | Match updater cadence.               |
| `LIVE_GAME_CACHE_UPDATE_INTERVAL` | seconds       |      30 | backend | Live cache refresh cadence.          |
| `LOL_PRO_TASK_ON_STARTUP`         | bool          | `false` | backend | Sync pro players on startup.         |
| `SITE_MAP_TASK_ON_STARTUP`        | bool          | `false` | backend | Generate sitemap on startup.         |
| `LETS_ENCRYPT_PATH`               | path          |       – | prod    | Directory with TLS certs/keys.       |

## Troubleshooting

* **Large WASM**: install `wasm-opt`; align `leptos` / `wasm-bindgen` / `cargo-leptos`.
* **Assets missing**: re-run the generator.
* **DB errors**: verify DSN or split vars; test with `psql`.
* **HTTP/3**: ensure **UDP:443** and `Alt-Svc`.
* **Certificates**: confirm `LETS_ENCRYPT_PATH` or explicit file paths.