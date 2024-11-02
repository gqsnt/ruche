FROM rustlang/rust:nightly-alpine AS builder

RUN apk update && \
    apk add --no-cache bash binaryen gcc git g++ libc-dev make npm protobuf-dev protoc gcompat libressl-dev


RUN curl --proto '=https' --tlsv1.2 -LsSf https://github.com/leptos-rs/cargo-leptos/releases/latest/download/cargo-leptos-installer.sh | sh

# Add the WASM target
RUN rustup target add wasm32-unknown-unknown
RUN cargo install cargo-leptos
RUN npm install -g tailwindcss

WORKDIR /work
COPY . .

RUN cargo leptos build --release -vv

FROM rustlang/rust:nightly-alpine AS runner

WORKDIR /app

COPY --from=builder /work/target/release/leptos-broken-gg /app/
COPY --from=builder /work/target/site /app/site
COPY --from=builder /work/Cargo.toml /app/

ENV RUST_LOG="info"
ENV LEPTOS_SITE_ADDR="0.0.0.0:8080"
ENV LEPTOS_SITE_ROOT=./site
EXPOSE 8080

CMD ["/app/leptos-broken-gg"]
