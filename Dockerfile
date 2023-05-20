FROM rustlang/rust:nightly

COPY . .
RUN chmod +x setup.sh
RUN ./setup.sh
CMD ["home_api_rust"]
EXPOSE 8000