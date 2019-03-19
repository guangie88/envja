FROM clux/muslrust:stable AS build

WORKDIR /build
COPY Cargo.lock Cargo.toml ./
COPY cli/Cargo.toml ./cli/
COPY lib/Cargo.toml ./lib/
RUN cargo fetch -v --locked

COPY cli/src ./cli/src
COPY lib/src ./lib/src
RUN cargo build --release -v --locked --all

FROM guangie88/releaser:alpine_upx-3_ghr-0.12 AS misc
WORKDIR /build
ARG ARCH=amd64
ARG OS=linux
COPY --from=build /build/target/x86_64-unknown-linux-musl/release/envja ./envja_${ARCH}_${OS}
RUN upx --best ./envja_${ARCH}_${OS}

FROM scratch AS release
WORKDIR /app
ARG ARCH=amd64
ARG OS=linux
COPY --from=misc /build/envja_${ARCH}_${OS} ./envja
CMD ./envja
