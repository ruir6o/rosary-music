FROM arm64v8/ubuntu:22.04
RUN apt-get update && apt-get install -y \
    libpq5 \
  && rm -rf /var/lib/apt/lists/*
COPY target/armv7-unknown-linux-gnueabihf/release/rosary-music /
ENTRYPOINT [ "/rosary-music" ]
