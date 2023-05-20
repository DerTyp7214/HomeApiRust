FROM rustlang/rust:nightly

COPY . .
RUN ./setup.sh
CMD ["home_api_rust"]
EXPOSE 8000