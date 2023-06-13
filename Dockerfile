FROM alpine:latest

COPY home_api_rust /usr/local/bin/home_api_rust
COPY migrations migrations
COPY diesel.toml diesel.toml
COPY diesel /usr/local/bin/diesel
CMD ["home_api_rust"]
EXPOSE 8000