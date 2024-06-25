FROM rust:slim AS builder

WORKDIR /src/builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev cmake

COPY . .
RUN --mount=type=cache,target=/src/builder/target/ cargo build --release && \
    cp target/release/musicbot /tmp/musicbot

FROM gcr.io/distroless/cc-debian12

WORKDIR /src/app

COPY --from=builder /tmp/musicbot .

CMD ["./musicbot"]
