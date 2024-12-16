<picture>
    <source srcset="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" media="(prefers-color-scheme: dark)">
    <img src="https://raw.githubusercontent.com/gqsnt/ruche/refs/heads/main/asset-generation/tmp/logo.png" alt="Ruche Logo">
</picture>

# Ruche: A High-Performance League of Legends Stats Platform
**Visit us at [ruche.lol](https://ruche.lol)**

Ruche is a cutting-edge League of Legends statistics platform designed for unparalleled speed and scalability. Inspired by industry leaders like OP.GG, Ruche offers comprehensive insights into summoner profiles, match histories, champion statistics, live games, and encounters—all delivered with exceptional performance.

## Key Features
### Summoner Profiles

- **Intuitive Search Functionality**
    - Search for summoners effortlessly using **Game Name**, **Tag Line**, and **Platform**.
    - If a summoner isn't found in the database, Ruche automatically fetches their data from Riot's API, adds them to the database, and redirects you to their profile.

- **Detailed Summoner Information**
    - **Profile Overview**
        - Displays summoner name, level, profile icon, and professional player status.
        - **Update Button**: Manually refresh the summoner's data and match history with a single click.
    - **Navigation Tabs**
        - **Matches**: Dive deep into match histories with advanced filtering and sorting.
        - **Champions**: View aggregated statistics for champions played.
        - **Encounters**: See which summoners you've played with or against most frequently.
        - **Live**: Access real-time game data if the summoner is currently in a game. A green indicator appears if the summoner is actively in a match.
        - **Encounter**: Analyze detailed stats for matches with a specific summoner.

### Match History

- **Comprehensive Match Cards**
    - **Summary Information**
        - Match outcome, KDA, kill participation, items, and a list of participants.
    - **Participant Details**
        - Encounter counts displayed as green tags for summoners you've played with or against.
        - Professional players highlighted with purple tags linking to their LolPros.gg profiles.
    - **Expandable Match Details**
        - **Overview Tab**: General stats and performance metrics.
        - **Team Tab**: In-depth breakdown of team compositions and stats (not implemented).
        - **Build Tab**: Timelines of item purchases, sales, skill upgrades, and perks.

- **Advanced Match Filters**
    - **Real-Time Filtering**
        - Filter matches by Champion, Queue Type, Start Date, and End Date without page reloads.
    - **Dynamic Updates**
        - Instantly update match lists and statistics based on selected filters.

### Champion Statistics
- **Detailed Performance Metrics**
    - Total matches played, wins, losses, win rate, average kills, deaths, assists, KDA ratio, gold earned, CS, damage dealt, damage taken, and multi-kill counts.
- **Advanced Sorting Options**
    - Sort champions by **Win Rate**, **Average KDA**, **Gold Earned**, **CS**, **Damage Dealt**, **Damage Taken**, and **Multi-Kills** to analyze performance effectively.

### Encounters

- **Encounter List**
    - Displays summoners you've frequently played with or against.
    - **Search Functionality**: Find specific summoners within your encounter list.
- **Encounter Details**
    - **With and Against Tabs**: Toggle between matches where you've played with or against a specific summoner.
    - **Statistical Comparison**
        - Side-by-side stats including total matches, wins, losses, win rates, average KDA, and kill participation.
    - **Match List with Filters**
        - View all shared matches with advanced filtering options similar to the match history.

### Live Games
- **Real-Time Game Information**
    - View current game mode, map, game length, and participant details if the summoner is in a live game.
- **'In Live Game' Indicator**
    - The "Live" tab on the summoner profile page now turns green when the summoner is actively in a match.
- **Automatic Refresh Interval**
    - Live game cache is automatically updated at intervals defined by `LIVE_GAME_CACHE_UPDATE_INTERVAL` in your `.env` file.
- **Refresh Button**
    - A refresh button on the live game tab lets you bypass the cache and fetch the latest data directly from the Riot API, ensuring real-time accuracy.
- **Participant Insights**
    - **Champion Picks**: See which champions are being played.
    - **Summoner Spells and Runes**: Detailed information on summoner spells and runes used.
    - **Player Statistics**
        - Encounter counts, ranked stats, and champion-specific stats for each participant.
- **Optimized Caching**
    - Live game data is cached for improved performance and reduced API calls.

### Real-Time Updates
- **Server-Sent Events (SSE)**
  - **Summoner Matches Events**: SSE event when match with related summoner is updated back-end, update matches, champions, and encounters.
  - **Live Game Status Events**: SSE event update ui and refresh live game page to reflect if a summoner is currently in a game, turning the "Live" navigation tab green.
  - **Debounce Mechanism:** Limits updates to once per 500ms to prevent client overload.
  - **Efficient Subscription Management**: Inactive SSE subscriptions are periodically cleaned up to free resources.

### Advanced Filtering and Sorting
- **Global Match Filters**
    - Available across Matches, Champions, Encounters, and Encounter pages.
    - Filters include **Champion**, **Queue Type**, **Start Date**, and **End Date**.
- **Dynamic Sorting**
    - Easily sort data to identify trends and analyze performance.

## Technical Highlights
### Frontend

- **Leptos Framework**
    - Utilizes Leptos, a reactive Rust framework with SSR and CSR capabilities.
    - **Component-Based Architecture**
        - Modular components like `MatchFilters`, `Pagination`, `SummonerNav`, and `MatchDetails` for maintainability and reusability.
    - **State Management**
        - Employs context providers and reactive signals for efficient state handling.
    - **Serialization/Deserialization**
        - Employs Bitcode, a lightweight serialization library optimized for speed and compression with zstd

### Backend

- **Rust-Powered Performance**
    - Built entirely in Rust for maximum efficiency and safety.
- **Asynchronous Operations**
    - Leverages Axum and Tokio for non-blocking, high-performance server operations.
- **Database Interaction**
    - Async support through SQLx and PostgreSQL for scalable data management.
- **Riot API Integration**
    - Utilizes Riven for seamless interaction with Riot's API.
- **Task Management**
    - Custom Task Director manages background tasks with precision and efficiency.
- **Robust Error Handling**
  - **Comprehensive Error Types (`AppError`)**
    - Detailed error handling covering database errors, API failures, parsing errors.
  - **Consistent Error Management**
    - Uniform error responses and logging for debugging and stability.


### Optimization Techniques
- **Asset Optimization**
    - Custom pipeline for asset management.
        - Downloads assets from Riot's API and Community Dragon.
        - Generates AVIF images and CSS sprites in AVIF format.
    - **In-Memory Asset Serving**
        - JS, CSS, WASM, and images are compressed and served from memory using `MemoryServe`.
- **Compression**
    - Utilizes Brotli and Zstd.
- **Database Efficiency**
    - Bulk inserts and updates minimize database load.
    - Optimized query and efficient index speed up data retrieval.

- **Caching Mechanisms with Thread-safe `DashMap`**
    - SSE broadcaster
    - Live Game Cache
- **Serialization/Deserialization**
    - Transitioned from serde to  rkyv to bitcode, favoring Bitcode for its size, performance and compatibility with zstd compression.

## Security and SEO
### Security Measures

- **TLS with Rustls**
    - Secure communication with modern TLS protocols.
- **Automatic HTTPS Redirection**
    - Ensures all traffic is encrypted by redirecting HTTP requests to HTTPS.
- **Environment Configuration**
    - Sensitive data managed through environment variables to enhance security.

### SEO Optimization

- **Dynamic Meta Tags**
    - Each summoner page includes titles and descriptions for better search visibility.
- **Clean and Canonical URLs**
    - Improves indexing and prevents duplicate content issues.
- **Sitemap Generation**
    - Automated daily updates to `sitemap.xml` for comprehensive search engine indexing.

## Architecture Highlights
### Server Functions
- **Data Retrieval and Updates**
    - Efficient fetching and updating of summoner data, matches, and live games.
    - Automatic handling of data inconsistencies and conflicts.
- **Task Management**
    - **Custom Task Scheduler**
        - Manages background tasks efficiently using a priority queue.
    - **Concurrent Execution Control**
        - Prevents race conditions and ensures data integrity.
    - **Background Tasks**
      - **Update Matches**: Fetches latest match details and resolves summoner conflicts.
      - **Update Pro Players**: Keeps professional player data up-to-date.
      - **Sitemap Generation**: Enhances SEO with daily sitemap updates.
      - **Clean SSE broadcaster Cache**: Maintains cache health for optimal performance.
      - **Handle Live Game Cache**: Manages live game data for real-time updates.
      
### Frontend Components
- **Reusable and Efficient**
    - Components designed for high reusability and minimal re-renders.
- **Reactive Design**
    - State changes propagate efficiently, providing a seamless user experience.



### Task Management
- **Custom Task Scheduler**
    - Manages background tasks efficiently using a priority queue.
- **Concurrent Execution Control**
    - Prevents race conditions and ensures data integrity.

### Asset Delivery
- **MemoryServe**
    - Serves assets directly from memory to reduce disk I/O.
- **Support for Compressed Assets**
    - Assets served with Brotli and Zstd compression.



## Installation
if you encounter problem with wasm , ensure wasm-* version are unique between ruche/Cargo.toml, cargo-leptos bin and leptos crate

### Requirements
- **Rust**: Install Rust using [rustup](https://rustup.rs/).
- **Nasm**: Install [Nasm](https://www.nasm.us/) Required for building `nasm-rs` used in `ravif` for AVIF encoding.
- **PostgreSQL**: Install PostgreSQL , add a ruche database, modify password and allow socket connection.
- **Riot API Key**: Obtain a Riot API key from the [Riot Developer Portal](https://developer.riotgames.com/).

### Setup
```bash
# Install Rust nightly
rustup default nightly;

# Add the WebAssembly target
rustup target add wasm32-unknown-unknown;

# Install wasm optimization tools/ run it if you encounter error with js `failed to grow table`
cargo install wasm-opt;

# Install cargo leptos with wasm-bindgen=0.2.97
cargo install --git https://github.com/gqsnt/cargo-leptos cargo-leptos;

# Clone the repository
git clone https://github.com/gqsnt/ruche.git;
cd ruche;

# Copy and configure the environment file
cp .env.example .env;

# Install tailwindcss
cd ruche;
npm i;
cd ..;
 
# Generate assets and css sprites add -- --help to see all options
cargo run --bin asset-generation --release;
```

### Local Development
```bash
# Check that ENV is set to DEV in .env
# Run leptos
cargo leptos watch;
```

### Production Build

#### Requirements
- **Let's Encrypt**: Install [Certbot](https://letsencrypt.org/) for TLS certificates.

#### Setup
The repository should be cloned on /etc path so default path is /etc/ruche

```bash
# kill nginx if running
sudo systemctl stop nginx;
sudo systemctl disable nginx;
killall nginx;

# Open firewall ports  80 and 443
sudo ufw allow 80;
sudo ufw allow 443;

# Add certifcate for domain and modify .env LETS_ENCRYPT_PATH with the site key/cert path
sudo certbot certonly --standalone -d `your_domain.com`;

# Copy service file to systemd
sudo cp /etc/ruche/ruche.service /etc/systemd/system/ruche.service;
# Enable the service
sudo systemctl enable ruche;

# git pull, download and rebuild assets if needed, rebuild the project, stop the service, make release  copy to ruche-release and start the service
sh rebuild.sh;
```


### Postgres Database Setup
```bash
# Create a new database
sudo -u postgres psql -c "CREATE DATABASE ruche;";
# Update the password in the .env file and in the database
sudo -u postgres psql -c "ALTER USER postgres PASSWORD 'password';";

# Allow socket connection in pg_hba.conf
# find the file location
sudo -u postgres psql -c "SHOW hba_file;";
# modify default local with METHOD trust
# local   all             all                                     trust
sudo nano /etc/postgresql/17/main/pg_hba.conf;
# Restart postgresql
sudo systemctl restart postgresql
```




## Contributing
### Contributions are welcome! Please follow these steps:
- **Fork** the repository.
- **Create a new branch** for your feature or bugfix.
- **Write clear, concise commit messages**.
- **Submit a pull request** detailing your changes.

## Acknowledgements
- **Riot Games**: For providing the API and resources necessary for data retrieval.
- **Leptos Framework**: For enabling a reactive frontend in Rust.
- **Open Source Community**: For the numerous libraries and tools that make this project possible.



## Future Roadmap
- **Implement Team Tab in Match Details**
  - Provide an in-depth breakdown of team compositions and stats.
- **Mobile Optimization**
    - Improve responsiveness and usability on mobile devices.
- **Improve Mark Branding**
    - Enhance the visual identity of Ruche with a unique and memorable brand.
