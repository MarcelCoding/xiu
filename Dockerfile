FROM rust:slim-buster as builder

WORKDIR /src

# TODO: improve caching
COPY . .

RUN cargo build --release --package xiu --bin xiu

FROM debian:11-slim

COPY --from=builder /src/target/release/xiu /xiu

ENTRYPOINT ["/xiu"]
