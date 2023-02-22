FROM rust:slim-buster as build

RUN apt-get update
RUN apt-get install build-essential libssl-dev pkg-config clang lld protobuf-compiler git -y
RUN rustup default nightly

# we need to do this due the requirement of the current git hash for fluvio-socket
RUN git init
RUN echo "dump" | tee temp
RUN git add temp
RUN git config --global user.email "dump"
RUN git config --global user.name "dump"
RUN git commit -m 'init'

COPY ./yaufs-control-plane ./yaufs-control-plane
COPY ./yaufs-common ./yaufs-common
COPY ./yaufs-proto ./yaufs-proto
COPY ./yaufs-codegen ./yaufs-codegen
COPY ./proto ./proto

WORKDIR ./yaufs-control-plane
RUN cargo build --release

FROM gcr.io/distroless/cc-debian11

COPY --from=build ./yaufs-control-plane/target/release/yaufs-control-plane .

ENTRYPOINT ["./yaufs-control-plane"]