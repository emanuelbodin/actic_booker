FROM ghcr.io/cargo-lambda/cargo-lambda:latest AS build

WORKDIR /build

# Compile dependencies only — this layer is reused until Cargo.toml / Cargo.lock change
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs \
    && cargo lambda build --release \
    && rm -rf src

# Rebuild the crate when source changes; deps stay cached in target/
COPY src ./src
RUN find src -name '*.rs' -exec touch {} + \
    && cargo lambda build --release

FROM public.ecr.aws/lambda/provided:al2023 AS runtime

COPY --from=build /build/target/lambda/actic-booker/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# Not used with custom runtime, but kept for info
CMD ["app.handler"]
