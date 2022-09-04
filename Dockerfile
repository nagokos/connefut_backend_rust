FROM rust:latest as builder

RUN USER=root cargo new --bin app

WORKDIR /app

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN apt-get update && \
  TZ=Asia/Tokyo apt-get install -y tzdata && \
  apt-get install -y \
  build-essential \
  git \
  clang \ 
  lld \
  cmake \
  libstdc++-10-dev \
  libxxhash-dev \
  zlib1g-dev \
  libssl-dev \
  build-essential \
  curl \
  && rm -rf /var/lib/apt/lists/*

RUN echo "install golang-migrate" \
    && curl -L -O https://github.com/golang-migrate/migrate/releases/download/v4.15.2/migrate.linux-amd64.tar.gz \
    && tar xf migrate.linux-amd64.tar.gz \
    && rm migrate.linux-amd64.tar.gz \
    && mv migrate /usr/local/bin \
    && echo "install tbls" \
    && curl -L -O https://github.com/k1LoW/tbls/releases/download/v1.56.1/tbls_v1.56.1_linux_amd64.tar.gz \
    && tar xf tbls_v1.56.1_linux_amd64.tar.gz \
    && rm tbls_v1.56.1_linux_amd64.tar.gz \
    && mv tbls /usr/local/bin 

RUN echo "git clone mold" \
    && git clone https://github.com/rui314/mold.git \
    && cd mold \
    && git checkout v1.4.1 \
    && make -j$(nproc) CXX=clang++ \
    && make install 

RUN echo "install rust tools" \
    && rustup component add rustfmt clippy \
    && cargo install cargo-watch cargo-edit cargo-audit \
    && cargo install --force cargo-make \
    && cargo install --locked cargo-outdated

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

FROM debian:buster-slim
RUN rm ./target/release/deps/connefut_api*
RUN cargo build --release
COPY --from=builder /app/target/release/connefut_api .
EXPOSE 8080
CMD ["./connefut_api"]
