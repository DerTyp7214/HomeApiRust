FROM rustlang/rust:nightly

COPY . .
RUN chmod +x setup.sh
RUN ./setup.sh

FROM alpine:latest

COPY --from=0 /home_api_rust /usr/local/bin/home_api_rust

CMD ["home_api_rust"]
EXPOSE 8000