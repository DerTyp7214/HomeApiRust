FROM alpine:latest

COPY target/release/home_api_rust /usr/local/bin/home_api_rust
COPY migrations migrations
COPY diesel.toml diesel.toml
RUN apk add --no-cache rust cargo
RUN cargo install diesel_cli --no-default-features --features postgres
CMD ["home_api_rust"]
EXPOSE 8000