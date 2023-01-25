FROM rust:1.66.1 as builder

RUN USER=root cargo new --bin fourth-ical-rs
WORKDIR ./fourth-ical-rs
# RUN --mount=type=cache,target=./target --mount=type=cache,target=/root/.cargo/registry
COPY ./Cargo.toml ./Cargo.toml
RUN cargo clean
RUN cargo build --release

RUN rm src/*.rs
ADD . ./
RUN rm ./target/release/deps/fourth_ical_rs*
RUN cargo build --release


FROM debian:bullseye-slim
ARG APP=/usr/src/app

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /fourth-ical-rs/target/release/fourth-ical-rs /fourth-ical-rs
EXPOSE 7878

CMD ["/fourth-ical-rs"]

# docker image build --platform linux/arm/v7 -t fourth-ical-rs .
# docker save -o fourth-ical-rs.tar fourth-ical-rs
# Host:
# docker load -i fourth-ical-rs.tar
