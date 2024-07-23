FROM debian:bookworm-slim AS runtime

COPY ./target/release/v8-test /usr/local/bin

ENTRYPOINT ["/usr/local/bin/v8-test"]
