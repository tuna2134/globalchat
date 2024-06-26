FROM rust:slim AS builder

WORKDIR /src/app

RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake

COPY . .
RUN --mount=type=cache,target=/src/app/target/ SQLX_OFFLINE=true cargo build --release && \
    cp target/release/globalchat /tmp/globalchat

FROM gcr.io/distroless/cc-debian12

WORKDIR /src/app

COPY --from=builder /tmp/globalchat .

CMD ["./globalchat"]
