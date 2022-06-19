FROM rust:slim AS builder

RUN update-ca-certificates

ENV USER=xiu
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /xiu

COPY . .
RUN cargo build --release --package xiu --bin xiu

FROM gcr.io/distroless/cc

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /xiu

COPY --from=builder /xiu/target/release/xiu ./xiu

USER xiu:xiu

CMD ["/xiu/xiu"]
