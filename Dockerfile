FROM scratch
COPY target/armv7-unknown-linux-musleabihf/release/rosary-music /
ENTRYPOINT [ "/rosary-music" ]
