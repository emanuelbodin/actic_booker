#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<EOF
Usage: $(basename "$0") <tag> <ecr_url>

Build the Docker image and push it to Amazon ECR.

Arguments:
  tag       Image tag (e.g. latest, v1.0.0)
  ecr_url   ECR repository URL, with or without a tag
            (e.g. 123456789012.dkr.ecr.eu-north-1.amazonaws.com/actic-booker)

Example:
  $(basename "$0") latest 123456789012.dkr.ecr.eu-north-1.amazonaws.com/actic-booker
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  usage
  exit 0
fi

if [[ $# -ne 2 ]]; then
  usage
  exit 1
fi

TAG="$1"
ECR_URL="$2"
ECR_REPO="${ECR_URL%%:*}"
REGISTRY="${ECR_REPO%%/*}"

if [[ ! "$REGISTRY" =~ ^[0-9]+\.dkr\.ecr\.([a-z0-9-]+)\.amazonaws\.com$ ]]; then
  echo "error: could not parse region from ECR URL: $ECR_REPO" >&2
  echo "expected format: <account>.dkr.ecr.<region>.amazonaws.com/<repository>" >&2
  exit 1
fi

REGION="${BASH_REMATCH[1]}"
IMAGE="${ECR_REPO}:${TAG}"

cd "$(dirname "${BASH_SOURCE[0]}")/.."

echo "Logging in to ECR registry ${REGISTRY} (${REGION})..."
aws ecr get-login-password --region "$REGION" | docker login --username AWS --password-stdin "$REGISTRY"

echo "Building ${IMAGE}..."
# --provenance=false is required: AWS Lambda does not support multi-platform images
docker build --provenance=false -t "$IMAGE" .

echo "Pushing ${IMAGE}..."
docker push "$IMAGE"

echo "Pushed ${IMAGE}"
