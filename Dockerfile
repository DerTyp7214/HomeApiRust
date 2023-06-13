FROM alpine:latest

COPY target/release/home_api_rust /usr/local/bin/home_api_rust
COPY migrations migrations
COPY diesel.toml diesel.toml
RUN apk add curl
RUN apk add gcc
RUN curl -proto '=https' -tlsv1.2 -sSf https://sh.rustup.rs | sh 
RUN cargo install diesel_cli --no-default-features --features postgres
CMD ["home_api_rust"]
EXPOSE 8000