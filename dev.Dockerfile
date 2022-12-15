FROM ubuntu:22.04
RUN apt update && apt install -y wget build-essential libpq-dev
RUN wget -O rustup-init.sh https://sh.rustup.rs \
 && chmod +x rustup-init.sh \
 && ./rustup-init.sh -y --profile minimal
WORKDIR /src
COPY . .
RUN . "$HOME/.cargo/env" \
 && cargo build --release
ENTRYPOINT [ "/src/target/release/rosary-music" ]
