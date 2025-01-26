# Introduction

actic-booker is a Rust project that implements an AWS Lambda function in Rust.

- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

## Building

Build for production by running `cargo lambda build --release`

## Testing

Regular tests are run with `cargo test`

## Local development

First, run `cargo lambda watch` to start a local server. When you make changes to the code, the server will automatically restart.

Second, invoke the lambda by running `cargo lambda invoke --data-file ./data.json`

## Deploying

To deploy the project, run `cargo lambda deploy`. This will create an IAM role and a Lambda function in your AWS account.
