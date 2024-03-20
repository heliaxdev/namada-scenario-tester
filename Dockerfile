# use the default dart image as the build image
FROM rust:1.76 AS builder

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

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates curl

WORKDIR /app

# copy the runtime files
COPY scenarios /app
COPY --from=builder /app/target/release/scenario-tester /app/scenario-tester 

# download masp parameters
RUN curl -o /app/masp-spend.params -L https://github.com/anoma/masp-mpc/releases/download/namada-trusted-setup/masp-spend.params\?raw\=true
RUN curl -o /app/masp-output.params -L https://github.com/anoma/masp-mpc/releases/download/namada-trusted-setup/masp-output.params?raw=true
RUN curl -o /app/masp-convert.params -L https://github.com/anoma/masp-mpc/releases/download/namada-trusted-setup/masp-convert.params?raw=true

ENV NAMADA_MASP_PARAMS_DIR /app

ENTRYPOINT ["./scenario-tester"]