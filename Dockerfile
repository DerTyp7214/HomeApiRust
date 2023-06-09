FROM alpine:latest

RUN ls -la .
RUN ls -la target
RUN ls -la target/release
COPY target/release/home_api_rust /usr/local/bin/home_api_rust
CMD ["home_api_rust"]
EXPOSE 8000