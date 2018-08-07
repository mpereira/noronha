FROM ekidd/rust-musl-builder:1.28.0 AS noronha-builder

WORKDIR /home/rust

COPY Cargo.toml Cargo.lock ./

RUN echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f src/main.rs

COPY config config
COPY src src

RUN cargo build --release

FROM alpine:3.8

WORKDIR /srv/noronha

COPY --from=noronha-builder /home/rust/config config
COPY --from=noronha-builder /home/rust/target/x86_64-unknown-linux-musl/release/noronha bin/noronha

CMD ["/srv/noronha/bin/noronha"]
