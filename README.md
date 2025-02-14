# Introduction

actic-booker is a Rust project that implements an AWS Lambda function in Rust.The function is intended to be invoked by an EventBridge to automatically book classes to avoid getting on the waiting list. Actic open registration to their classes 10 days in advanced, by running an Eventbridge rule as a cron every day at a specific time. So if you for example want to book spinning classes at 18:45 on Mondays, you can schedule a cron 18:45 every day.

The syntax for the event can be found in `event.json`:

```sh
{
  "center_id": 110,
  "name": "Spinning",
  "day": "Mon",
  "start_time": "18:45"
}
```

This event will make the lambda book you on all spinning classes at Mondays at 18:45 in gym with center_id `110`. A list with all gym center_id's can be found in `centers.json`. In `classes.json` some activities are listed, these may vary depending on gym.

You also need to provide your actic credentials as env variables.

More info:

- [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html)

## Building

Build for production by running `cargo lambda build --release`

To build docker image run: `docker build -t actic_booker:latest --provenance=false .`
Test the docker image locally with `docker run -p 8080:8080 --env-file .env  actic_booker:latest` and invoke with by running:

```sh
curl -X POST --data '{"version":"2.0","center_id":110,"name":"Spinning","day":"Mon","start_time":"18:45","requestContext":{"http":{"method":"GET"},"timeEpoch":0}}' http://localhost:8080/2015-03-31/functions/function/invocations
```

## Local development

```sh
cat .env.example > .env
```

and entter your username and password in the .env file.

Then, run `cargo lambda watch` to start a local server. When you make changes to the code, the server will automatically restart.

Invoke the lambda by running `cargo lambda invoke --data-file ./event.json`

## Deploying

First create terraform remote backend:

```sh
cd terraform/backend-state
touch .terraform.tfvars // fill this with proper values
terraform init
terraform apply
```

Create necessary resources in AWS:

```sh
cd terraform/app
touch .terraform.tfvars // fill this with proper values
terraform init -backend-config="backend.hcl"
terraform apply
```

Push docker image:

```sh
aws ecr get-login-password --region <region> | docker login --username AWS --password-stdin <ecr_repo_url>
docker push <ecr_repo_url>:<tag>
```

Mkae sure you have authenticated to you AWS account.

To deploy the lambda, run `cargo lambda deploy`. This will create an IAM role and a Lambda function in your AWS account. You may need to create an IAM policy to give the Eventbridge rule permission to invoke the Lambda.
