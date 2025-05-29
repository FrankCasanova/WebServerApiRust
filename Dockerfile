# syntax=docker/dockerfile:1
FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM rust:latest as runtime
WORKDIR /app
RUN apt-get update && apt-get install -y libpq-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/webserver /app/webserver
COPY src/templates ./templates
COPY migrations ./migrations
COPY .env .env
COPY diesel.toml diesel.toml
ENV TEMPLATES_PATH=/app/templates
EXPOSE 8080
CMD ["/app/webserver"]
