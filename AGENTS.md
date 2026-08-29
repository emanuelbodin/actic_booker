# AGENTS.md

Guidance for coding agents working in this repository.

## Project

actic-booker is a Rust AWS Lambda that books Actic gym classes via `https://webapi.actic.se`. EventBridge cron rules fire ~10 days before a class (when registration opens). The production image is a custom Lambda runtime (`public.ecr.aws/lambda/provided:al2023`) built with Cargo Lambda.

## Layout

- `src/main.rs` — Lambda entry (`service_fn(function_handler)`)
- `src/event_handler.rs` — event payload + orchestration
- `src/credentials.rs` — `USERNAME`/`PASSWORD` from env, else SSM (`{prefix}/{user}/username|password`)
- `src/actic.rs` — Actic HTTP client (login, list classes, list bookings, book)
- `event.json` — sample EventBridge payload
- `centers.json` / `classes.json` — gym IDs and activity names (activities vary by gym)
- `Dockerfile` — multi-stage Cargo Lambda build
- `scripts/build-and-push-ecr.sh` — build and push to ECR
- `terraform/backend-state` — remote state bucket
- `terraform/app` — ECR, Lambda, IAM, EventBridge

## Event payload

```json
{
  "user": "kristian",
  "center_id": 110,
  "name": "Spinning",
  "day": "Mon",
  "start_time": "18:45",
  "latest": "true"
}
```

- `user` selects which SSM credential pair to load (ignored when env credentials are set).
- `day` is chrono weekday (`Mon`, `Tue`, …), matched against class date.
- `latest: "true"` skips classes fewer than 7 days away so only the newly opened (~10-day) slot is booked.

## Commands

Local (requires [Cargo Lambda](https://www.cargo-lambda.info/guide/installation.html) and `.env` from `.env.example`):

```sh
cargo lambda watch
cargo lambda invoke --data-file ./event.json
cargo test
```

The integration test in `event_handler` is `#[ignore]` (needs Actic credentials and network). Do not un-ignore it in CI.

Docker / ECR (Lambda rejects multi-platform images; needs Docker with Buildx):

```sh
docker build -t actic_booker:latest --provenance=false .
./scripts/build-and-push-ecr.sh <tag> <ecr_url>
```

Terraform: `terraform/backend-state` first, then `terraform/app` with `terraform init -backend-config="backend.hcl"`. Never commit `*.tfvars`, `.env`, or state files.

## Conventions

- Keep Actic API field names (`startTime`, `bookingIdCompound`, `accessToken`) in serde `rename`s; do not “fix” them to snake_case in JSON.
- Prefer `tracing` for Lambda logs; existing `println!` in `actic.rs` is legacy—do not expand that pattern.
- Map HTTP/client errors into `lambda_runtime::Error` at the handler boundary.
- Docker builds must use `--provenance=false`.
- Credentials: local `.env` (`USERNAME`, `PASSWORD`); production SSM under `SSM_PARAMETER_PREFIX` (default `/actic-booker`). Never log passwords or tokens.

## Do not

- Commit secrets or terraform var files.
- Call Actic booking APIs from tests or scripts unless the user explicitly wants a real booking.
- Change Lambda/EventBridge/ECR wiring without updating `terraform/app` to match.
