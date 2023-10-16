# use the default dart image as the build image
FROM rust:1.70 AS builder

# copy the current folder into the build folder
COPY . /app

# set the work directory
WORKDIR /app

RUN DEBIAN_FRONTEND=noninteractive apt-get update

# build app
RUN cargo build --release

# use a slim image
FROM debian:bullseye-slim

RUN DEBIAN_FRONTEND=noninteractive apt-get update && apt-get install -y ca-certificates

# copy the runtime files
COPY scenarios /app
COPY --from=builder /app/target/release/namada-load-tester /app/namada-load-tester 
WORKDIR /app

# start the dart webserver
ENTRYPOINT ["./namada-load-tester"]
CMD ["--help"]