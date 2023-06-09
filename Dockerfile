FROM alpine:latest

COPY target/release/home_api_rust /usr/local/bin/home_api_rust
CMD ["home_api_rust"]
EXPOSE 8000