FROM ghcr.io/cross-rs/armv7-unknown-linux-gnueabihf

RUN dpkg --add-architecture armhf
RUN apt-get update && apt-get -y install libpq-dev:armhf

ENV PKG_CONFIG_PATH="/usr/lib/arm-linux-gnueabihf/pkgconfig/:${PKG_CONFIG_PATH}"
