FROM rust:latest AS builder
RUN update-ca-certificates

# Create appuser
ENV USER=murkov
ENV UID=10001

RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /murkov

COPY ./ .

RUN cargo build --release

FROM ubuntu:20.04

RUN apt install -y libopus-dev
RUN apt -y install cmake protobuf-compiler essential autoconf automake libtool m4 ffmpeg
RUN add-apt-repository --yes ppa:tomtomtom/yt-dlp
RUN apt update
RUN apt install -y yt-dlp

COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group

WORKDIR /murkov

# Copy build
COPY --from=builder /murkov/target/release/murkov ./

# Use an unprivileged user.
USER murkov:murkov