FROM rust:1.50 as builder
WORKDIR /usr/src/repoutil
COPY . .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/repoutil /usr/local/bin/repoutil
CMD ["repoutil"]
