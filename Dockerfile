FROM rust:slim-buster as builder

WORKDIR /src

# TODO: improve caching
COPY . .

RUN cargo build --release

FROM debian:11-slim

COPY --from=builder /src/target/release/xiu /xiu

ENTRYPOINT ["xiu"]
