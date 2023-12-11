# use the default dart image as the build image
FROM rust:1.70 AS builder

# copy the current folder into the build folder
COPY . /app

# set the work directory
WORKDIR /app

RUN apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install --no-install-recommends --assume-yes \
    libprotobuf-dev \
    build-essential \
    clang-tools-11 \
    git \
    libssl-dev \
    pkg-config \
    protobuf-compiler \
    libudev-dev \
    && apt-get clean

# build app
RUN cargo build --release

# use a slim image
FROM debian:bullseye-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates

# copy the runtime files
COPY scenarios /app
COPY --from=builder /app/target/release/namada-scenario-tester /app/namada-scenario-tester 
WORKDIR /app

# start the dart webserver
ENTRYPOINT ["./namada-scenario-tester"]
CMD ["--help"]