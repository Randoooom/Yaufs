FROM busybox:latest as template-service-preperation

COPY ./yaufs-template-service/Cargo.toml ./Cargo.toml
RUN sed -i '3s/.*/version = "0.1.0"/' Cargo.toml

FROM rust:slim-buster as template-service-build

RUN apt-get update
RUN apt-get install build-essential libssl-dev pkg-config clang lld protobuf-compiler -y
RUN rustup default nightly

# compile dependencies
RUN cargo new --bin yaufs-template-service
WORKDIR ./yaufs-template-service
COPY --from=template-service-preperation Cargo.toml ./Cargo.toml
COPY ./yaufs-monitoring ../yaufs-monitoring
COPY ./yaufs-tonic ../yaufs-tonic
RUN cargo build --release

# build the lib
RUN rm -rf ./src/*
RUN rm -rf ./target/release/deps/yaufs_template_service*
COPY ./yaufs-template-service/Cargo.toml ./Cargo.toml
COPY ./yaufs-template-service/src ./src
COPY ./yaufs-template-service/build.rs ./build.rs
COPY ./proto ../proto

RUN cargo build --release

FROM gcr.io/distroless/cc-debian11 as template-service

COPY --from=template-service-build ./yaufs-template-service/target/release/yaufs-template-service .

ENTRYPOINT ["./yaufs-template-service"]
