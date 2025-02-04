FROM ghcr.io/cargo-lambda/cargo-lambda:latest AS build

WORKDIR /build
COPY . .
RUN cargo lambda build --release

FROM public.ecr.aws/lambda/provided:al2023 AS runtime

COPY --from=build /build/target/lambda/actic-booker/bootstrap ${LAMBDA_RUNTIME_DIR}/bootstrap

# Not used with custom runtime, but kept for info
CMD ["app.handler"]