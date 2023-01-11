#FROM busybox:latest as monitoring-preperation
#
#COPY ./yaufs-monitoring/Cargo.toml ./Cargo.toml
#RUN sed -i '3s/.*/version = "0.1.0"/' Cargo.toml
#
#FROM rust:slim-buster as monitoring-build
#
#RUN apt-get update
#RUN apt-get install build-essential libssl-dev pkg-config clang lld protobuf-compiler -y
#RUN rustup default nightly
#
#
## compile dependencies
#RUN cargo new --lib yaufs-monitoring
#WORKDIR ./yaufs-monitoring
#COPY --from=monitoring-preperation Cargo.toml ./Cargo.toml
#RUN cargo build --release
#
## build the lib
#RUN rm -rf ./src/*
#RUN rm -rf ./target/release/deps/yaufs-monitoring*
#COPY ./yaufs-monitoring/src ./src
#
#RUN cargo build --release

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
RUN cargo build --release

# build the lib
RUN rm -rf ./src/*
RUN rm -rf ./target/release/deps/yaufs_template_service*
COPY ./yaufs-template-service/Cargo.toml ./Cargo.toml
COPY ./yaufs-template-service/src ./src
COPY ./yaufs-template-service/build.rs ./build.rs
COPY ./proto ../proto

RUN cargo build --release

FROM debian:buster-slim as template-service

RUN apt-get update
RUN apt-get install libssl-dev ca-certificates -y

COPY --from=template-service-build ./yaufs-template-service/target/release/yaufs-template-service .

ENTRYPOINT ["./yaufs-template-service"]
