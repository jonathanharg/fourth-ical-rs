FROM clux/muslrust:stable AS builder
COPY Cargo.* .
COPY src/*.rs src/
RUN --mount=type=cache,target=/volume/target \
    --mount=type=cache,target=/root/.cargo/registry \
    cargo build --release --bin fourth-ical-rs && \
    mv /volume/target/x86_64-unknown-linux-musl/release/fourth-ical-rs .

FROM gcr.io/distroless/static:nonroot
COPY --from=builder --chown=nonroot:nonroot /volume/fourth-ical-rs /app/
EXPOSE 8787
ENTRYPOINT ["/app/fourth-ical-rs"]