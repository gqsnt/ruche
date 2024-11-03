FROM rustlang/rust:nightly-bullseye AS builder

# If you’re using stable, use this instead
# FROM rust:1.70-bullseye as builder

# Install cargo-binstall, which makes it easier to install other
# cargo extensions like cargo-leptos
RUN wget https://github.com/cargo-bins/cargo-binstall/releases/latest/download/cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN tar -xvf cargo-binstall-x86_64-unknown-linux-musl.tgz
RUN cp cargo-binstall /usr/local/cargo/bin

RUN apt-get update && apt-get install -y \
    curl

RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash -
RUN apt-get install -y nodejs

# Install cargo-leptos
RUN cargo binstall cargo-leptos -y
#RUN cargo install cargo-leptos

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown

# Make an /app dir, which everything will eventually live in
RUN mkdir -p /app
WORKDIR /app
COPY . .

# ls the directory to make sure everything is there
RUN ls

RUN npm install -D tailwindcss

# Build the app
RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-bullseye AS runner
# Copy the server binary to the /app directory
COPY --from=builder /app/target/release/leptos-broken-gg /app/
# /target/site contains our JS/WASM/CSS, etc.
COPY --from=builder /app/target/site /app/site
# Copy Cargo.toml if it’s needed at runtime
COPY --from=builder /app/Cargo.toml /app/
WORKDIR /app

# Set any required env variables and
ENV RUST_LOG="info"
ENV APP_ENVIRONMENT="production"
ENV LEPTOS_SITE_ADDR="0.0.0.0:3000"
ENV LEPTOS_SITE_ROOT="site"
ENV RUST_BACKTRACE=1
EXPOSE 3000
# Run the server
CMD ["/app/leptos-broken-gg"]