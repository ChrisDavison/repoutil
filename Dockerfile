FROM rust:1.52-slim-buster as builder
WORKDIR /usr/src/repoutil
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/repoutil /usr/local/bin/repoutil
ENTRYPOINT ["repoutil"]
